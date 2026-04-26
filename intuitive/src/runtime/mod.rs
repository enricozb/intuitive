//! The runtime: event loop, mounted-element tree, signal storage.
//!
//! The runtime owns per-mounted-element state keyed by [`ElementKey`].
//! Each mounted element is a [`Node`] (internal) holding:
//!
//! - a (shared) handle to the render closure taken from the [`Element`] value
//!   that mounted it,
//! - its composed children (the `Vec<Element>` from the parent's `ui!{}` `{ ...
//!   }` block — the cheap-Clone snapshot the parent supplied), and
//! - a record of which children it placed on its most recent render (used to GC
//!   unmounted children on reconcile).
//!
//! See `design.md` for the high-level rendering protocol.

use std::{
  collections::{HashMap, HashSet},
  io,
  sync::Arc,
};

use futures::StreamExt;
use tokio::{
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
  task::AbortHandle,
};

pub mod context;
pub mod element;

pub use context::Context;
pub use element::Element;

use crate::{
  buffer::{Buffer, Cell},
  event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind},
  geometry::{Point, Rectangle, Size},
  runtime::{context::Placement, element::RenderFn},
  signal::{ErasedSignal, SignalId, SlotId},
  terminal::Terminal,
};

// -----------------------------------------------------------------------------
// Identity
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Span {
  pub file: &'static str,
  pub line: u32,
  pub column: u32,
}

impl Span {
  #[must_use]
  pub fn new(file: &'static str, line: u32, column: u32) -> Self {
    Self { file, line, column }
  }

  /// A sentinel span used as the default before `ui!{}` stamps a
  /// real one onto an `Element`. Distinct from any real span so
  /// failures-to-stamp surface clearly.
  #[must_use]
  pub fn placeholder() -> Self {
    Self {
      file: "<unstamped>",
      line: 0,
      column: 0,
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ElementKey {
  pub parent: Option<Arc<ElementKey>>,
  pub span: Span,
  pub user: Option<String>,
}

impl ElementKey {
  /// The synthetic key for the root element.
  #[must_use]
  pub fn root() -> Self {
    Self {
      parent: None,
      span: Span {
        file: "<root>",
        line: 0,
        column: 0,
      },
      user: None,
    }
  }

  #[must_use]
  pub fn child(parent: &ElementKey, span: Span, user: Option<String>) -> Self {
    Self {
      parent: Some(Arc::new(parent.clone())),
      span,
      user,
    }
  }
}

// -----------------------------------------------------------------------------
// Events
// -----------------------------------------------------------------------------

/// Internal events processed by the runtime's main loop.
#[derive(Debug)]
pub enum Event {
  /// First event after `render_root` is called: triggers the initial paint.
  Init,
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(Size),
  /// Notification that a signal's storage was just mutated. The runtime
  /// walks the signal's subscriber set and marks them dirty.
  Signal(SignalId),
  /// Quit the loop.
  Exit,
}

// -----------------------------------------------------------------------------
// Hook slot bookkeeping
// -----------------------------------------------------------------------------

/// Per-element table of key handlers, keyed by hook call site.
pub(crate) type KeyHandlerSlots = HashMap<SlotId, Box<dyn Fn(KeyEvent)>>;

/// Per-element table of click handlers, keyed by hook call site. A
/// click handler fires when a mouse button is pressed *and* the
/// element's current rect contains the click position. The runtime
/// hit-tests against `rects[key]` before invoking the handler, so
/// the handler itself doesn't have to.
pub(crate) type ClickHandlerSlots = HashMap<SlotId, Box<dyn Fn(MouseEvent)>>;

/// Slots touched by the currently-rendering element. After the render
/// closure returns, untouched slots in each hook table are dropped
/// (signals are released, handlers are removed, tasks are aborted).
///
/// Saved/restored across nested renders so each element gets its own.
#[derive(Default)]
pub(crate) struct Touched {
  pub signals: HashSet<SlotId>,
  pub key_handlers: HashSet<SlotId>,
  pub click_handlers: HashSet<SlotId>,
  pub tasks: HashSet<SlotId>,
}

// -----------------------------------------------------------------------------
// Mounted-element state
// -----------------------------------------------------------------------------

/// One child placement remembered from the previous render. Used to
/// GC children that were placed last time but not this time.
#[derive(Clone)]
pub(crate) struct RecordedPlacement {
  pub child_key: ElementKey,
}

/// The mounted state for one element, keyed by its [`ElementKey`].
pub(crate) struct Node {
  /// Shared handle to the element's render closure. Cloned (cheap
  /// Arc bump) into a local before invocation so the runtime can
  /// release its borrow on the node and reborrow `self` mutably
  /// during the render.
  pub(crate) render: RenderFn,
  /// Composed children snapshot supplied at mount time. Cloned
  /// per-placement to produce the Element handed to the runtime
  /// for mounting; the original entry stays here for re-mounting
  /// later (e.g. when a `Select`-style parent toggles which child
  /// is active).
  pub(crate) composed: Vec<Element>,
  /// Children placed during the most recent render of this element.
  /// Used to GC keys no longer placed.
  pub(crate) last_placements: Vec<RecordedPlacement>,
  /// Layering depth assigned to this element by its parent at
  /// placement time. Inherited from the parent by default (via
  /// [`Context::render`]); `ZStack` and other overlap-producing
  /// containers assign distinct z's per child (via
  /// [`Context::render_with_z`]).
  ///
  /// Used by [`Runtime::render_dirty`] to (a) determine which
  /// other mounted elements potentially overlay this one and need
  /// to be repainted after this element re-renders, and (b) order
  /// the dirty drain so lower-z elements are painted first.
  ///
  /// [`Context::render`]: crate::runtime::Context::render
  /// [`Context::render_with_z`]: crate::runtime::Context::render_with_z
  pub(crate) z: i32,
  /// Whether this element fully paints every cell of its assigned
  /// rect. Set by the render closure via
  /// [`Context::set_opaque`] (defaults to `false` at the start of
  /// each render).
  ///
  /// The runtime uses this to decide, after partially re-rendering
  /// a lower-z element, which higher-z elements need to be repainted
  /// on top: only opaque elements whose rect intersects the dirty
  /// rect are added back to the dirty set. Transparent elements
  /// (pure layout containers, `Text`, etc.) are skipped — their
  /// cells inside the dirty rect either weren't painted by them
  /// or come from a descendant that the overlap walk will pick up
  /// separately.
  ///
  /// [`Context::set_opaque`]: crate::runtime::Context::set_opaque
  pub(crate) opaque: bool,
}

// -----------------------------------------------------------------------------
// Runtime
// -----------------------------------------------------------------------------

pub struct Runtime {
  pub(crate) front: Buffer,
  pub(crate) back: Buffer,

  pub(crate) nodes: HashMap<ElementKey, Node>,
  pub(crate) rects: HashMap<ElementKey, Rectangle>,

  // Per-element hook tables. Each is keyed by SlotId (call site) and
  // follows the same "touched-or-dropped" lifecycle: the rendering
  // element marks slots it visits, untouched slots are GC'd at
  // end-of-render.
  pub(crate) signals: HashMap<ElementKey, HashMap<SlotId, ErasedSignal>>,
  pub(crate) key_handlers: HashMap<ElementKey, KeyHandlerSlots>,
  pub(crate) click_handlers: HashMap<ElementKey, ClickHandlerSlots>,
  pub(crate) tasks: HashMap<ElementKey, HashMap<SlotId, AbortHandle>>,

  pub(crate) subs_by_element: HashMap<ElementKey, Vec<SignalId>>,
  pub(crate) subs_by_signal: HashMap<SignalId, HashSet<ElementKey>>,

  pub(crate) dirty: HashSet<ElementKey>,

  /// Sender given to every `Signal<T>` handle so mutations can enqueue
  /// themselves on the same channel as input events.
  pub(crate) event_tx: UnboundedSender<Event>,

  /// Slots touched by the currently-rendering element. Swapped in/out
  /// across nested renders by `Runtime::render_one`.
  pub(crate) touched: Touched,

  /// Opacity flag for the currently-rendering element. Flipped to
  /// `true` by `Context::set_opaque` during the render closure;
  /// committed onto the corresponding `Node` at end-of-render.
  /// Saved/restored across nested renders alongside `touched`.
  pub(crate) current_opaque: bool,
}

impl Runtime {
  fn new(size: Size, event_tx: UnboundedSender<Event>) -> Self {
    Self {
      front: Buffer::new(size),
      back: Buffer::new(size),
      nodes: HashMap::new(),
      rects: HashMap::new(),
      signals: HashMap::new(),
      key_handlers: HashMap::new(),
      click_handlers: HashMap::new(),
      tasks: HashMap::new(),
      subs_by_element: HashMap::new(),
      subs_by_signal: HashMap::new(),
      dirty: HashSet::new(),
      event_tx,
      touched: Touched::default(),
      current_opaque: false,
    }
  }

  /// Mark the currently-rendering element as opaque (i.e. it fully
  /// paints every cell of its assigned rect). Called from
  /// [`Context::set_opaque`]. The flag is committed onto the
  /// element's `Node` at end-of-render.
  ///
  /// [`Context::set_opaque`]: crate::runtime::Context::set_opaque
  pub(crate) fn set_current_opaque(&mut self) {
    self.current_opaque = true;
  }

  pub(crate) fn event_tx(&self) -> &UnboundedSender<Event> {
    &self.event_tx
  }

  // ---------------------------------------------------------------------------
  // Public entry point
  // ---------------------------------------------------------------------------

  /// Run the event loop until `Event::Exit` is received.
  ///
  /// `root` is the top-level element (produced by `ui!{}` or a
  /// hand-rolled `Element::new(..)`). The terminal is taken into raw +
  /// alternate screen mode for the duration. Background tasks should
  /// be spawned by components via `ctx.spawn(future)`.
  ///
  /// # Errors
  ///
  /// Returns any I/O error from entering raw mode, querying the
  /// terminal size, or flushing output during the event loop.
  pub async fn render_root(root: Element) -> io::Result<()> {
    let mut term = Terminal::enter()?;
    let size = term.size()?;
    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let mut rt = Self::new(size, tx.clone());
    let root_key = ElementKey::root();
    let rect = rt.back.rect();
    rt.mount_at(root_key.clone(), root, rect, 0);
    rt.dirty.insert(root_key);

    // Kick off with an initial render.
    let _ = tx.send(Event::Init);

    // Translate crossterm input events to our `Event`. Run on a background
    // task so the main loop can select between this and SignalChanged
    // events through the same mpsc.
    spawn_input_pump(tx.clone());

    rt.run_loop(rx, &mut term).await
  }

  // ---------------------------------------------------------------------------
  // Headless test API
  // ---------------------------------------------------------------------------

  /// Construct a runtime and event channel for headless tests. The
  /// caller is responsible for driving events.
  #[doc(hidden)]
  #[must_use]
  pub fn new_headless(size: Size) -> (Self, UnboundedSender<Event>, UnboundedReceiver<Event>) {
    let (tx, rx) = mpsc::unbounded_channel::<Event>();
    let rt = Self::new(size, tx.clone());
    (rt, tx, rx)
  }

  /// Mount an element as root.
  #[doc(hidden)]
  pub fn mount_root(&mut self, root: Element) {
    let root_key = ElementKey::root();
    let rect = self.back.rect();
    self.mount_at(root_key.clone(), root, rect, 0);
    self.dirty.insert(root_key);
  }

  /// Process a single event (no flush). Useful for tests.
  #[doc(hidden)]
  pub fn step(&mut self, event: Event) -> bool {
    match event {
      Event::Init => {}
      Event::Key(k) => self.dispatch_key(k),
      Event::Mouse(m) => self.dispatch_mouse(m),
      Event::Resize(s) => {
        // Resize only `back`. We deliberately leave `front` at its old
        // size so the next diff sees a size mismatch and triggers a
        // full repaint (with a clear-screen first). This avoids stale
        // content in cells that grew into view, where neither our
        // `front` nor the terminal can be trusted.
        self.back.resize(s);
        if let Some(rect) = self.rects.get_mut(&ElementKey::root()) {
          *rect = self.back.rect();
        }
        self.dirty.insert(ElementKey::root());
      }
      Event::Signal(id) => self.notify_signal_changed(&id),
      Event::Exit => return false,
    }
    self.render_dirty();
    true
  }

  /// Get a reference to the back buffer (for inspection in tests).
  #[doc(hidden)]
  #[must_use]
  pub fn back_buffer(&self) -> &Buffer {
    &self.back
  }

  async fn run_loop(&mut self, mut rx: UnboundedReceiver<Event>, term: &mut Terminal) -> io::Result<()> {
    while let Some(event) = rx.recv().await {
      match event {
        Event::Init => { /* drain dirty below paints initial frame */ }
        Event::Key(k) => self.dispatch_key(k),
        Event::Mouse(m) => self.dispatch_mouse(m),
        Event::Resize(s) => {
          // See `step` above: only `back` is resized so the next diff
          // is a full repaint.
          self.back.resize(s);
          if let Some(rect) = self.rects.get_mut(&ElementKey::root()) {
            *rect = self.back.rect();
          }
          self.dirty.insert(ElementKey::root());
        }
        Event::Signal(id) => self.notify_signal_changed(&id),
        Event::Exit => break,
      }
      self.render_dirty();
      term.flush(&self.front, &self.back)?;
      // After flushing, front matches back for next diff.
      self.front = self.back.clone();
    }
    Ok(())
  }

  // ---------------------------------------------------------------------------
  // Mount / reconcile
  // ---------------------------------------------------------------------------

  /// Mount-or-reconcile an `Element` value under `key`. If a Node
  /// already exists at `key`, its `render` Arc handle and `composed`
  /// vec are replaced from the supplied element (its identity-metadata
  /// — span + `user_key` — match by construction, since `key` was
  /// derived from them). Otherwise a fresh Node is inserted.
  ///
  /// In either case, the rect and z are updated and the key is
  /// marked dirty.
  pub(crate) fn mount_at(&mut self, key: ElementKey, elem: Element, rect: Rectangle, z: i32) {
    self.rects.insert(key.clone(), rect);
    let Element {
      span: _,
      user_key: _,
      children,
      render,
    } = elem;
    if let Some(node) = self.nodes.get_mut(&key) {
      node.render = render;
      node.composed = children;
      node.z = z;
      // `last_placements` is *not* reset here: it's overwritten when
      // this node re-renders.
      // `opaque` is overwritten at end-of-render by
      // `render_one`, so we leave it untouched here.
    } else {
      self.nodes.insert(
        key.clone(),
        Node {
          render,
          composed: children,
          last_placements: Vec::new(),
          z,
          opaque: false,
        },
      );
    }
    self.dirty.insert(key);
  }

  fn render_dirty(&mut self) {
    // Drain into an owned set so we release the borrow on self.dirty.
    // Re-renders may add more keys to `dirty` (e.g. when a parent
    // re-render mounts new children, or when the overlap walk
    // below promotes overlay elements to dirty), so loop until the
    // queue is empty.
    while !self.dirty.is_empty() {
      // Sort the current batch by ascending z so a lower-z element
      // is repainted *before* any higher-z overlay we promote in
      // response. Without this, two simultaneously-dirty elements
      // at different z's could be processed in `HashSet` order and
      // produce wrong z-stacking (the higher-z element renders
      // first, then the lower-z element clears its rect and paints
      // over it). Stable sort by (z, key as tiebreaker) keeps the
      // order deterministic across runs.
      let mut batch: Vec<ElementKey> = std::mem::take(&mut self.dirty).into_iter().collect();
      batch.sort_by(|a, b| {
        let za = self.nodes.get(a).map_or(0, |n| n.z);
        let zb = self.nodes.get(b).map_or(0, |n| n.z);
        za.cmp(&zb)
      });
      for key in batch {
        // Skip if this element was unmounted since being marked dirty.
        if self.nodes.contains_key(&key) {
          // Top-level re-render: clear the dirty element's rect before
          // re-painting it (so old content disappears even if the
          // closure now draws nothing). Descendants don't re-clear:
          // their rects sit inside this rect, which we just blanked,
          // and re-clearing per-descendant would erase overlapping
          // siblings (e.g. `ZStack` overlays).
          //
          // After re-rendering, promote any opaque mounted
          // elements with `z > z(key)` whose rect intersects
          // `rects[key]` back to dirty, so they repaint on top of
          // the cells we just stamped. They get picked up by the
          // next iteration of this `while` loop (in z-sorted
          // order). See `promote_overlays` for the rationale.
          self.render_one(&key, true);
          self.promote_overlays(&key);
        }
      }
    }
  }

  /// After re-rendering `key` (which cleared and repainted its
  /// rect), find every other mounted element that needs to redraw
  /// on top to restore correct z-stacking, and add it to
  /// `self.dirty`.
  ///
  /// An element `O` needs to redraw on top iff:
  ///
  /// - `O != key` and `O` is currently mounted,
  /// - `O` is *opaque* (every cell of its rect belongs to it; see
  ///   `Context::set_opaque`),
  /// - `O`'s `z` is strictly greater than `key`'s `z`, and
  /// - `O`'s rect intersects `key`'s rect.
  ///
  /// Transparent elements are skipped because they only write the
  /// cells they explicitly painted last frame, which either (a) lie
  /// outside `key`'s rect (no overlap), or (b) belong to one of
  /// their *descendants* that has its own rect — and any opaque
  /// descendant satisfying the predicate gets added to dirty by
  /// its own entry in the walk. Concretely: a `ZStack { HStack {
  /// Timer, Tasks }, Border { Input } }` only promotes `Border`
  /// (opaque) and skips the `ZStack` / `Centered` wrappers, even
  /// though those wrappers' rects also cover the timer.
  fn promote_overlays(&mut self, key: &ElementKey) {
    let Some(dirty_rect) = self.rects.get(key).copied() else {
      return;
    };
    let dirty_z = self.nodes.get(key).map_or(0, |n| n.z);
    // Snapshot the candidate keys before mutating `self.dirty` so we
    // don't hold a borrow on `nodes` across the insert.
    let overlays: Vec<ElementKey> = self
      .nodes
      .iter()
      .filter_map(|(k, node)| {
        if k == key || !node.opaque || node.z <= dirty_z {
          return None;
        }
        let rect = self.rects.get(k)?;
        rect.intersects(dirty_rect).then(|| k.clone())
      })
      .collect();
    for k in overlays {
      self.dirty.insert(k);
    }
  }

  /// Render a single element by key. Briefly takes the node out so
  /// the render closure can be invoked with a `Context` that holds
  /// `&mut Runtime`. After the closure returns, drains placements
  /// and dispatches each child mount/reconcile.
  ///
  /// `clear_rect` controls whether this call blanks the element's
  /// rect before invoking its render closure. The dirty drain passes
  /// `true` (to support transparent re-renders — cells the new render
  /// no longer touches end up blank instead of leaking the previous
  /// frame). Placement-driven recursive calls pass `false`, because
  /// the ancestor that started this paint already cleared the
  /// covering rect and re-clearing per-child would wipe overlapping
  /// earlier siblings (`ZStack`-style overlays).
  pub(crate) fn render_one(&mut self, key: &ElementKey, clear_rect: bool) {
    let Some(mut node) = self.nodes.remove(key) else {
      return;
    };

    // Save parent state we need to restore after this render.
    let prev_touched = std::mem::take(&mut self.touched);
    // Reset the opacity flag at the start of each render. The
    // closure may call `Context::set_opaque` to flip it to true;
    // we commit the post-render value onto `node.opaque` below.
    // The previous value is restored after we return so a nested
    // child render doesn't clobber the parent's in-flight flag.
    let prev_opaque = std::mem::replace(&mut self.current_opaque, false);

    // Clear this element's rect before invoking its render closure.
    // Components are responsible only for drawing cells with actual
    // content; any cells they don't touch end up blank. This is what
    // makes "transparency" work for components like `Empty` and for
    // conditional children that switch from a drawing component to a
    // non-drawing one.
    //
    // Skipped when this render was triggered by a parent's placement
    // loop (`clear_rect == false`): the parent already cleared the
    // covering rect, and re-clearing here would erase any earlier
    // sibling drawings that this element should be drawing over
    // (the `ZStack` overlay case).
    if clear_rect && let Some(rect) = self.rects.get(key).copied() {
      self.back.region(rect).fill(Cell::BLANK);
    }

    let mut placements: Vec<Placement> = Vec::new();
    let render_handle = Arc::clone(&node.render);
    let composed = std::mem::take(&mut node.composed);
    let node_z = node.z;
    {
      let mut ctx = Context::new(self, key.clone(), &composed, &mut placements, node_z);
      let mut render = render_handle
        .lock()
        .expect("render closure mutex poisoned (a previous render panicked)");
      (render)(&mut ctx);
    }
    // Restore composed onto the node before continuing.
    node.composed = composed;
    // Commit the opacity flag observed during this render onto the
    // node, then restore the parent's in-flight flag.
    node.opaque = std::mem::replace(&mut self.current_opaque, prev_opaque);

    // GC any hook slots not touched this render.
    let touched = std::mem::replace(&mut self.touched, prev_touched);
    self.gc_untouched_slots(key, &touched);

    // Process placements: each is a child to mount under `key`.
    //
    // Placement *order* is the user's painting order: `ZStack { A; B }`
    // requires B to draw over A, layout containers may expect that
    // overlapping siblings (e.g. modal overlays) honor declaration
    // order, and component authors generally expect later
    // `ctx.render(..)` calls to draw on top of earlier ones.
    //
    // We therefore (a) record each placement on the parent for future
    // diffing, and (b) render each placed child *synchronously, in
    // declaration order*, before returning. Rendering synchronously
    // here means we don't have to rely on the dirty-set drain (which
    // is `HashSet`-ordered and therefore unspecified) to preserve
    // z-order. The dirty set is still used to *trigger* re-renders
    // from signal mutations and the initial mount; once a re-render
    // starts at some node, its subtree is painted in placement
    // order from this loop.
    let mut new_placements: Vec<RecordedPlacement> = Vec::with_capacity(placements.len());
    let prev_placements = std::mem::take(&mut node.last_placements);

    // Re-insert the node before recursing into children so that
    // grandchildren can still observe `key`'s tables (e.g. for
    // `ctx.composed()` walks of ancestors, future hook lookups).
    node.last_placements = Vec::new(); // populated below
    self.nodes.insert(key.clone(), node);

    for placement in placements {
      let Placement { elem, rect, z } = placement;
      let child_key = ElementKey::child(key, elem.span, elem.user_key.clone());
      new_placements.push(RecordedPlacement {
        child_key: child_key.clone(),
      });
      self.mount_at(child_key.clone(), elem, rect, z);
      // Remove from `dirty`: we're about to render synchronously, and
      // we don't want the outer drain loop to re-render this child a
      // second time at an unspecified moment.
      self.dirty.remove(&child_key);
      self.render_one(&child_key, false);
    }

    // GC children no longer placed.
    let live: HashSet<&ElementKey> = new_placements.iter().map(|p| &p.child_key).collect();
    for prev in &prev_placements {
      if !live.contains(&prev.child_key) {
        self.gc_subtree(&prev.child_key);
      }
    }

    // Commit the recorded placement list onto the (now re-inserted)
    // node.
    if let Some(n) = self.nodes.get_mut(key) {
      n.last_placements = new_placements;
    }
  }

  /// Drop hook slots in `key`'s tables that were not visited during
  /// this render. Tasks are aborted; signals' subscribers are
  /// also walked off the reverse index when a signal is removed.
  fn gc_untouched_slots(&mut self, key: &ElementKey, touched: &Touched) {
    if let Some(slots) = self.signals.get_mut(key) {
      let stale: Vec<SlotId> = slots.keys().filter(|s| !touched.signals.contains(s)).copied().collect();
      for slot in stale {
        slots.remove(&slot);
        // Also unsubscribe this element from the signal we're
        // dropping. The signal's id is `(element=key, slot)`.
        let id = SignalId {
          element: key.clone(),
          slot,
        };
        if let Some(set) = self.subs_by_signal.get_mut(&id) {
          set.remove(key);
        }
        if let Some(deps) = self.subs_by_element.get_mut(key) {
          deps.retain(|sig_id| sig_id != &id);
        }
      }
    }
    if let Some(slots) = self.key_handlers.get_mut(key) {
      slots.retain(|s, _| touched.key_handlers.contains(s));
    }
    if let Some(slots) = self.click_handlers.get_mut(key) {
      slots.retain(|s, _| touched.click_handlers.contains(s));
    }
    if let Some(slots) = self.tasks.get_mut(key) {
      let stale: Vec<SlotId> = slots.keys().filter(|s| !touched.tasks.contains(s)).copied().collect();
      for slot in stale {
        if let Some(handle) = slots.remove(&slot) {
          handle.abort();
        }
      }
    }
  }

  fn gc_subtree(&mut self, key: &ElementKey) {
    // Recurse into the node's last_placements before removing it.
    let kids: Vec<ElementKey> = self
      .nodes
      .get(key)
      .map(|n| n.last_placements.iter().map(|p| p.child_key.clone()).collect())
      .unwrap_or_default();
    for k in &kids {
      self.gc_subtree(k);
    }
    self.nodes.remove(key);
    self.rects.remove(key);
    self.signals.remove(key);
    self.key_handlers.remove(key);
    self.click_handlers.remove(key);
    self.dirty.remove(key);
    // Abort any tasks spawned by this element.
    if let Some(tasks) = self.tasks.remove(key) {
      for (_, handle) in tasks {
        handle.abort();
      }
    }
    if let Some(subs) = self.subs_by_element.remove(key) {
      for sig in subs {
        if let Some(set) = self.subs_by_signal.get_mut(&sig) {
          set.remove(key);
        }
      }
    }
  }

  // ---------------------------------------------------------------------------
  // Event dispatch
  // ---------------------------------------------------------------------------

  fn dispatch_key(&mut self, k: KeyEvent) {
    // Built-in: Esc / Ctrl-C quits.
    if k.kind == KeyEventKind::Press
      && (k.code == KeyCode::Esc || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL)))
    {
      let _ = self.event_tx.send(Event::Exit);
      return;
    }
    // Walk every element, invoking every key handler registered on it.
    // Handlers are `Fn`, so we call them through a shared reference.
    // Order is unspecified across elements and across slots.
    let keys: Vec<ElementKey> = self.key_handlers.keys().cloned().collect();
    for key in keys {
      if let Some(slots) = self.key_handlers.get(&key) {
        for handler in slots.values() {
          handler(k);
        }
      }
    }
  }

  /// Dispatch a mouse event to `on_click` handlers.
  ///
  /// Only fires on mouse-button press (`MouseEventKind::Down(_)`);
  /// drag / up / motion / scroll events are ignored. The runtime
  /// hit-tests each registered handler against its element's
  /// current rect, and invokes only those whose rect contains the
  /// click position. Multiple handlers may fire from a single
  /// click — e.g. when a parent and a child both register
  /// `on_click`, both fire because the child's rect (and therefore
  /// the click) lies inside the parent's. Order across elements
  /// and across slots is unspecified.
  fn dispatch_mouse(&mut self, m: MouseEvent) {
    if !matches!(m.kind, MouseEventKind::Down(_)) {
      return;
    }
    let p = Point::new(m.column, m.row);
    let keys: Vec<ElementKey> = self.click_handlers.keys().cloned().collect();
    for key in keys {
      let Some(rect) = self.rects.get(&key).copied() else {
        continue;
      };
      if !rect.contains(p) {
        continue;
      }
      if let Some(slots) = self.click_handlers.get(&key) {
        for handler in slots.values() {
          handler(m);
        }
      }
    }
  }

  fn notify_signal_changed(&mut self, id: &SignalId) {
    if let Some(subs) = self.subs_by_signal.get(id) {
      for k in subs.iter().cloned().collect::<Vec<_>>() {
        self.dirty.insert(k);
      }
    }
  }
}

// -----------------------------------------------------------------------------
// Input pump: convert crossterm events into our `Event` variants.
// -----------------------------------------------------------------------------

impl Drop for Runtime {
  fn drop(&mut self) {
    // Abort all spawned tasks so they don't outlive the runtime.
    for (_, tasks) in self.tasks.drain() {
      for (_, handle) in tasks {
        handle.abort();
      }
    }
  }
}

fn spawn_input_pump(tx: UnboundedSender<Event>) {
  tokio::spawn(async move {
    let mut events = crossterm::event::EventStream::new();
    while let Some(Ok(ev)) = events.next().await {
      let our = match ev {
        crossterm::event::Event::Key(k) => Event::Key(k),
        crossterm::event::Event::Mouse(m) => Event::Mouse(m),
        crossterm::event::Event::Resize(w, h) => Event::Resize(Size::new(w, h)),
        // Focus / paste events not used in v1.
        _ => continue,
      };
      if tx.send(our).is_err() {
        break;
      }
    }
  });
}
