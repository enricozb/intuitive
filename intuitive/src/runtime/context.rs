//! `Context`: the per-render handle a component author interacts with.
//!
//! A `Context` is constructed by the runtime each time it invokes an
//! element's render closure. It carries:
//!
//! - the `ElementKey` of the element being rendered (for hook identity,
//!   signal-subscription bookkeeping, etc.),
//! - a [`Region`] view of the element's assigned rect in the back buffer (so
//!   the closure can draw), and
//! - an accumulator for *child placements* — the closure tells the runtime
//!   which Elements to render and where, either by referencing a composed-child
//!   index or by handing over a freshly-constructed dynamic Element value.
//!
//! After the closure returns, the runtime drains the accumulator and
//! recursively renders each placed child into its assigned rect.

use std::{
  future::Future,
  panic::Location,
  sync::{Arc, RwLock},
};

use crate::{
  buffer::Region,
  event::{KeyEvent, MouseEvent},
  geometry::Rectangle,
  runtime::{ElementKey, Runtime, element::Element},
  signal::{ErasedSignal, Signal, SignalId, SlotId},
};

/// A request, issued by the rendering closure, to mount-and-render
/// some child element into a sub-rect. Accumulated on `Context` during
/// the render and drained by the runtime afterward.
///
/// Composed and dynamic children flow through the same shape: once
/// the closure has pulled them out (`composed[idx].clone()` for
/// composed, a fresh value for dynamic), they're indistinguishable.
///
/// `z` is the layering depth assigned by the parent. By default
/// children inherit the parent's z (via [`Context::render`]); a
/// parent that produces overlapping siblings (e.g. `ZStack`) must
/// assign distinct z's via [`Context::render_with_z`] so the
/// runtime can preserve declared paint order even when only a
/// lower-z descendant is re-rendered alone.
pub(crate) struct Placement {
  pub elem: Element,
  pub rect: Rectangle,
  pub z: i32,
}

/// Per-render handle. See module docs.
pub struct Context<'a> {
  pub(crate) rt: &'a mut Runtime,
  pub(crate) current_key: ElementKey,
  /// Read-only handle to this element's composed children. The
  /// runtime owns the underlying `Vec<Element>` and grants the
  /// Context a borrow for the duration of the render. The closure
  /// body inspects (via `composed_len` / `composed`) and places
  /// (via `render_composed`); under the hood, `render_composed`
  /// clones the chosen child rather than moving it, so the
  /// composed-vec stays intact across re-renders.
  pub(crate) composed: &'a [Element],
  /// Accumulator for child placements made during this render.
  pub(crate) placements: &'a mut Vec<Placement>,
  /// Layering depth assigned to this element by its parent. New
  /// child placements default to this z (via [`Context::render`]);
  /// containers that produce overlapping siblings can assign a
  /// different z per child via [`Context::render_with_z`].
  pub(crate) z: i32,
}

impl<'a> Context<'a> {
  pub(crate) fn new(
    rt: &'a mut Runtime,
    current_key: ElementKey,
    composed: &'a [Element],
    placements: &'a mut Vec<Placement>,
    z: i32,
  ) -> Self {
    Self {
      rt,
      current_key,
      composed,
      placements,
      z,
    }
  }

  #[must_use]
  pub fn current_key(&self) -> &ElementKey {
    &self.current_key
  }

  /// The drawable region for this element.
  ///
  /// # Panics
  ///
  /// Panics if the runtime has no rectangle registered for the
  /// currently-rendering element. This indicates a runtime bug.
  pub fn region(&mut self) -> Region<'_> {
    let rect = *self.rt.rects.get(&self.current_key).expect("rect not registered");
    self.rt.back.region(rect)
  }

  /// Read-only view of this element's composed children — the
  /// `Vec<Element>` the parent supplied at the `ui!{}` call site's
  /// `{ ... }` block. The runtime owns the underlying vec.
  ///
  /// Components that declare a `#[children] children: Vec<Element>`
  /// parameter get a fresh `Vec<Element>` (cloned from this slice)
  /// bound as a local at the top of their render closure, by the
  /// `#[component]` macro. Hand-rolled components can call this
  /// directly and clone individual entries as needed.
  #[must_use]
  pub fn composed(&self) -> &[Element] {
    self.composed
  }

  /// This element's current z (layering depth), set by the parent
  /// at placement time. Children placed via [`Context::render`]
  /// inherit this value. See [`Context::render_with_z`] for the
  /// override.
  #[must_use]
  pub fn z(&self) -> i32 {
    self.z
  }

  /// Declare that this element fully paints every cell of its
  /// assigned rect on this render. Marking an element opaque lets
  /// the runtime's partial-re-render logic know that when a
  /// lower-z element underneath this one is repainted alone, the
  /// overlapping cells of *this* element must be repainted on top.
  ///
  /// Components that fill their region (e.g. `Border` with its
  /// background blank, `Input`, `Text` when it draws every cell)
  /// should call this. Pure layout containers (`HStack`, `VStack`,
  /// `ZStack`, `Centered`, `Empty`) should not.
  ///
  /// Like other hooks, this is render-scoped: a component that
  /// only sometimes draws an opaque background can call this
  /// conditionally, and the opacity flag is reset at the start of
  /// each render.
  pub fn set_opaque(&mut self) {
    self.rt.set_current_opaque();
  }

  /// Render a child Element into `rect`. The runtime reconciles
  /// under an `ElementKey` derived from `elem.span` +
  /// `elem.user_key`.
  ///
  /// This is the most common placement primitive: composed
  /// children are rendered by cloning the desired entry out of
  /// `ctx.composed()` (or, more typically, out of a `#[children]
  /// children: Vec<Element>` parameter that the `#[component]`
  /// macro materialized) and passing it here. Dynamic children
  /// constructed by the body (e.g. via inline `ui!{}`) are passed
  /// in directly.
  ///
  /// The child inherits this element's [`Context::z`]. Use
  /// [`Context::render_with_z`] to place a child at a distinct
  /// layering depth (e.g. a `ZStack` placing each child one level
  /// higher than the last).
  pub fn render(&mut self, elem: Element, rect: Rectangle) {
    let z = self.z;
    self.placements.push(Placement { elem, rect, z });
  }

  /// Like [`Context::render`], but places the child at an explicit
  /// z (layering depth) instead of inheriting this element's z.
  /// Used by containers that intentionally produce overlapping
  /// siblings (notably `ZStack`).
  pub fn render_with_z(&mut self, elem: Element, rect: Rectangle, z: i32) {
    self.placements.push(Placement { elem, rect, z });
  }

  /// Install a key handler at this call site. Like other hooks, the
  /// handler is keyed by `Location::caller()`: registering at the same
  /// source location across renders replaces the closure (with fresh
  /// captures); a call site not visited on a given render is dropped at
  /// end-of-render. Multiple `on_key` calls at different source
  /// locations register independent handlers, all of which fire for
  /// every key event.
  #[track_caller]
  pub fn on_key<F>(&mut self, f: F)
  where
    F: Fn(KeyEvent) + 'static,
  {
    let loc = Location::caller();
    let slot = SlotId(loc);
    self.rt.touched.key_handlers.insert(slot);
    self
      .rt
      .key_handlers
      .entry(self.current_key.clone())
      .or_default()
      .insert(slot, Box::new(f));
  }

  /// Install a click handler at this call site.
  ///
  /// The handler fires when a mouse button is pressed *and* the
  /// click's position is inside this element's current
  /// [`Rectangle`]. The runtime performs the hit-test before
  /// invoking the closure, so the handler body doesn't need to.
  /// Drag, release, motion, and scroll events are not delivered
  /// through `on_click`.
  ///
  /// A single click may fire multiple handlers when several
  /// elements' rects all contain the click position. This is the
  /// common case for nested layouts: if both a parent and a child
  /// register `on_click`, a click inside the child's rect fires
  /// both handlers (the child's rect is contained in the parent's,
  /// so the parent's hit-test also succeeds). Order between
  /// elements is unspecified — handlers should be written to
  /// commute, or to bail out based on element-local state.
  ///
  /// Like other hooks, the handler is keyed by
  /// `Location::caller()`: registering at the same source location
  /// across renders replaces the closure (with fresh captures);
  /// a call site not visited on a given render is dropped at
  /// end-of-render.
  ///
  /// [`Rectangle`]: crate::geometry::Rectangle
  #[track_caller]
  pub fn on_click<F>(&mut self, f: F)
  where
    F: Fn(MouseEvent) + 'static,
  {
    let loc = Location::caller();
    let slot = SlotId(loc);
    self.rt.touched.click_handlers.insert(slot);
    self
      .rt
      .click_handlers
      .entry(self.current_key.clone())
      .or_default()
      .insert(slot, Box::new(f));
  }

  /// Subscribe the currently-rendering element to a signal. Idempotent.
  pub(crate) fn subscribe(&mut self, id: SignalId) {
    let elem = self.current_key.clone();
    let set = self.rt.subs_by_signal.entry(id.clone()).or_default();
    if set.insert(elem.clone()) {
      // First time this element subscribed to this signal; record the
      // reverse index too.
      self.rt.subs_by_element.entry(elem).or_default().push(id);
    }
  }

  /// Look up or create a signal slot at this call site, defaulting to
  /// `T::default()` on first render.
  #[track_caller]
  pub fn new_signal_default<T>(&mut self) -> Signal<T>
  where
    T: Default + Send + Sync + 'static,
  {
    let loc = Location::caller();
    self.new_signal_inner::<T>(SlotId(loc), T::default)
  }

  /// Look up or create a signal slot, with an explicit value used
  /// only on first render.
  #[track_caller]
  pub fn new_signal<T>(&mut self, value: T) -> Signal<T>
  where
    T: Send + Sync + 'static,
  {
    let loc = Location::caller();
    self.new_signal_inner::<T>(SlotId(loc), move || value)
  }

  /// Look up or create a signal slot, with an explicit initializer used
  /// only on first render.
  #[track_caller]
  pub fn new_signal_with<T>(&mut self, init: impl FnOnce() -> T) -> Signal<T>
  where
    T: Send + Sync + 'static,
  {
    let loc = Location::caller();
    self.new_signal_inner::<T>(SlotId(loc), init)
  }

  fn new_signal_inner<T>(&mut self, slot: SlotId, init: impl FnOnce() -> T) -> Signal<T>
  where
    T: Send + Sync + 'static,
  {
    let elem = self.current_key.clone();
    let id = SignalId {
      element: elem.clone(),
      slot,
    };

    // Mark this slot as touched so it survives end-of-render GC.
    self.rt.touched.signals.insert(slot);

    // Look up existing storage; if present, reuse the same Arc so all
    // handles point at the same cell. Otherwise create a fresh one.
    let slots = self.rt.signals.entry(elem).or_default();
    let arc_any: ErasedSignal = if let Some(existing) = slots.get(&slot) {
      Arc::clone(existing)
    } else {
      let cell: Arc<RwLock<T>> = Arc::new(RwLock::new(init()));
      let erased: ErasedSignal = cell.clone();
      slots.insert(slot, erased);
      cell as Arc<dyn std::any::Any + Send + Sync>
    };

    // Downcast back to the concrete Arc<RwLock<T>>.
    let cell: Arc<RwLock<T>> = arc_any
      .downcast::<RwLock<T>>()
      .expect("signal type mismatch in new_signal");

    Signal::from_parts(id, cell, self.rt.event_tx().clone())
  }

  /// Spawn a background task tied to the current element's lifetime.
  ///
  /// Keyed by call site (`#[track_caller]`): the task is spawned on the
  /// first render that reaches this call site for this element. On
  /// subsequent renders the existing task is left running (the future
  /// passed in is dropped without being polled). The task is aborted
  /// when the element unmounts, when this call site is no longer
  /// visited (slot dropped at end-of-render), or when the runtime is
  /// dropped.
  ///
  /// Tasks typically read state via `Signal::peek` and write via
  /// `Signal::set` / `Signal::update`. Both work from any thread because
  /// signals are `Send + Sync`.
  #[track_caller]
  pub fn spawn<F>(&mut self, fut: F)
  where
    F: Future<Output = ()> + Send + 'static,
  {
    let loc = Location::caller();
    let slot = SlotId(loc);
    self.rt.touched.tasks.insert(slot);
    let tasks = self.rt.tasks.entry(self.current_key.clone()).or_default();
    if tasks.contains_key(&slot) {
      // Task already spawned for this element at this call site;
      // nothing to do. `fut` is dropped without being polled.
      return;
    }
    let join = tokio::spawn(fut);
    tasks.insert(slot, join.abort_handle());
  }
}
