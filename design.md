# Intuitive — Design

A component-based TUI framework, drawing on React (functional components,
hooks), SolidJS (signals with fine-grained dependency tracking), and SwiftUI
(declarative DSL).

This document describes the runtime, the rendering model, and the public API.
The macros (`#[component]`, `ui!{ .. }`) are sketched at the end and detailed
separately in `macros.md`.

## Mental model

- A **component** is a user-defined function (e.g. `fn Root(ctx: &mut
  Context) { .. }`). It is a definition — it exists once, in the source.
- An **element** is a runtime value produced by invoking a component at a
  particular call site with particular props. Concretely, an element is a
  `Box<dyn FnMut(&mut Context)>` closure that, when called, runs the
  component body. The `ui!{ .. }` macro is what turns `Foo(prop: x)`
  syntax into an element. One component can produce many elements (e.g.
  one per loop iteration).
- A program is a tree of **elements** at runtime. When the runtime renders
  an element, it invokes the closure with a `Context`. The component body
  uses that context to access its state slots and signals, register event
  handlers, register child elements (which the runtime will render into
  sub-regions), and draw into its assigned region.
- Layout is **parent-determined**: a parent assigns a `Rectangle` to each
  child element before the runtime renders it. Children do not influence
  their own size.
- Rendering is driven by an **event loop** in `Runtime`. Initial render
  happens on `Event::Init`. Subsequent renders are caused by signal
  mutations during event handlers, which mark elements dirty and trigger a
  re-render pass at the end of the event.

## Identity: `ElementKey` and `SlotId`

Every element in the tree has a stable `ElementKey`, composed of:
- the parent's `ElementKey`,
- the **call site** in source (`file!()`, `line!()`, `column!()`), emitted
  by the `ui!{ .. }` macro into the generated element constructor, and
- an optional user-supplied `key:` prop (required by convention for
  elements in a loop, where call sites alone collide).

The span and the user key are **combined**, not alternatives. A user-key
only disambiguates among siblings at the same call site (e.g. iterations
of one loop); it never causes elements at *different* call sites to be
treated as the same. Concretely, two `Task(key: "foo")` invocations on
different source lines produce different `ElementKey`s, because their
`SpanId`s differ.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Span {
  file:   &'static str,   // from file!()
  line:   u32,            // from line!()
  column: u32,            // from column!()
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ElementKey {
  parent: Option<Rc<ElementKey>>,
  span:   Span,
  /// User-supplied `key:` prop.
  user:   Option<String>,
}
```

Hooks (signals, state, effects) created inside a component body are
identified within an element by a `SlotId` derived from
`std::panic::Location::caller()` on the hook constructor. This uses
`#[track_caller]` and requires no proc macro — calling `ctx.new_signal()`
from two distinct source locations gives two distinct slots automatically.

```rust
/// A hook slot's identity within an element. Wraps the
/// `&'static Location` returned by `Location::caller()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SlotId(&'static std::panic::Location<'static>);
```

**Slot stability invariant:** each `SlotId` (i.e. each hook call site)
encountered in a render is stable per element — the same slot identity is
reused across renders. However, *which* slots an element touches on a
given render may legitimately differ, since hook constructors live at
stable call sites but may be guarded by conditionals over props or state.
Slots not touched on a re-render are dropped. (Concretely: this means
the state behind a `new_signal()` call inside an `if` branch is
discarded when control flow takes the other branch.)

**Subscription stability invariant:** for a given element, each
`Signal::get` call site must read the *same* `Signal` across every
render of that element. In other words, a child element receives a
fixed set of signals over its lifetime; a parent must not pass
`signal_A` on one render and `signal_B` on the next render to the same
child. Users who want conditional reactivity should branch at the
*element* level instead (different call sites → different elements →
independent subscription sets). The runtime can enforce this in debug
builds by recording `(ElementKey, get_call_site) → SignalId` and
panicking on mismatch.

This invariant lets the runtime treat subscriptions as **permanent**
until an element unmounts: there is no per-render subscription
clear-and-rebuild step. Subscription cleanup happens once, at unmount.

## Runtime

```rust
pub struct Runtime {
  // Event source. crossterm events plus framework-internal Init/Exit.
  events: EventStream,

  // Front-buffer (last flushed to terminal) and back-buffer (being drawn).
  // Diffed at the end of each render pass to emit escape sequences.
  front: Buffer,
  back:  Buffer,

  // Per-element state, keyed by ElementKey. `elements` stores the boxed
  // closures (one per mounted element); `signals` retains the
  // type-erased shared storage cells so they outlive the component
  // body's local handles.
  elements: HashMap<ElementKey, Element>,
  rects:    HashMap<ElementKey, Rectangle>,
  children: HashMap<ElementKey, Vec<ElementKey>>,
  signals:  HashMap<ElementKey, HashMap<SlotId, Arc<dyn Any + Send + Sync>>>,
  handlers: HashMap<ElementKey, Handlers>,   // on_key, on_mouse, etc.
  // Per-element table of background tasks spawned via `ctx.spawn`.
  // Aborted on element unmount and on runtime drop.
  tasks:    HashMap<ElementKey, HashMap<SlotId, AbortHandle>>,

  // Sender for in-process events. Cloned into every Signal<T> handle so
  // they can post `Event::SignalChanged(id)` notifications after
  // mutating storage.
  event_tx: EventSender,

  // Reverse index: for each element, which signals it is subscribed to.
  // Used at unmount to remove the element from each signal's subscriber
  // set in O(deps) rather than O(all signals).
  subs_by_element: HashMap<ElementKey, Vec<SignalId>>,
  // Forward index: for each signal, the elements that depend on it.
  // Used by `Signal::set` to enqueue subscribers into `dirty`.
  subs_by_signal:  HashMap<SignalId, HashSet<ElementKey>>,

  // Re-render queue. Drained after each event handler.
  dirty: HashSet<ElementKey>,
}

impl Runtime {
  pub fn new() -> Self { /* ... */ }

  pub async fn render_root(mut self, root: Element) -> Result<()> {
    let root_key = ElementKey::root();
    self.insert_element(root_key.clone(), root, self.back.full_rect());

    while let Some(event) = self.events.next().await {
      match event {
        Event::Init             => self.render_dirty(/* all */),
        Event::Key(k)           => self.dispatch_key(k),
        Event::Mouse(m)         => self.dispatch_mouse(m),
        Event::SignalChanged(i) => self.notify_signal_changed(i),
        Event::Resize(s)        => {
          // Resize only `back`. `front` is deliberately left at its
          // old size so the next `Buffer::diff` sees a size mismatch
          // and reports every cell of `back` as changed; `flush()`
          // then issues a clear-screen + full repaint. This is the
          // only way to safely fill cells that just grew into view,
          // since neither our stale `front` nor the terminal itself
          // can be trusted for them.
          self.back.resize(s);
          self.rects.insert(ElementKey::root(), self.back.full_rect());
          self.mark_dirty(ElementKey::root());
        }
        Event::Exit             => break,
      }
      self.render_dirty();
      self.flush();   // diff back vs front, emit escapes, swap.
    }
    Ok(())
  }

  /// Mark every element subscribed to `id` as dirty. Called from the
  /// event loop after a `Signal::set` / `Signal::update` posts an
  /// `Event::SignalChanged(id)` notification. Storage was already
  /// mutated synchronously by the signal; this loop pass only handles
  /// dirty propagation and re-rendering.
  fn notify_signal_changed(&mut self, id: SignalId) {
    if let Some(subs) = self.subs_by_signal.get(&id) {
      for k in subs.iter().cloned().collect::<Vec<_>>() {
        self.dirty.insert(k);
      }
    }
  }

  /// Insert an element into the runtime's tables and queue it for
  /// rendering. Used both for the root (from `render_root`) and for
  /// children (from `Context::render`).
  fn insert_element(&mut self, key: ElementKey, element: Element, rect: Rectangle) {
    self.elements.insert(key.clone(), element);
    self.rects.insert(key.clone(), rect);
    self.dirty.insert(key);
  }

  fn render_dirty(&mut self) {
    // `mem::take` swaps in an empty HashSet, releasing the borrow on
    // `self.dirty` so we can call `self.render` (which needs &mut self)
    // inside the loop.
    for key in std::mem::take(&mut self.dirty) {
      self.render(key);
    }
  }

  fn render(&mut self, key: ElementKey) {
    // Snapshot the previous child set before re-rendering. The new
    // child set is populated by ctx.render(..) calls inside the
    // closure below.
    let old_children = self.children.remove(&key).unwrap_or_default();

    // 1. Set up Context with this element's rect, hook map, and current_key.
    // 2. Invoke the element's closure. It will:
    //    - call ctx.new_signal()/new_state() to look up or create slots,
    //    - call signal.get(ctx), which subscribes (idempotently — the
    //      subscription set is stable for a given element by invariant),
    //    - register child elements via ctx.render(rect, child), which
    //      transitively renders them and populates self.children[key],
    //    - register handlers via ctx.on_key(..), etc.
    // 3. Drop any hook slots that were not touched this render.

    // 4. Diff old vs new child sets. Any child that was present last
    //    render but not this render has been unmounted; recursively
    //    GC its subtree.
    let new_children = self.children.get(&key).cloned().unwrap_or_default();
    for old in old_children {
      if !new_children.contains(&old) {
        self.gc_subtree(old);
      }
    }
  }

  /// Recursively remove an element and all its descendants from every
  /// runtime table, including unsubscribing from any signals via
  /// `subs_by_element`.
  fn gc_subtree(&mut self, key: ElementKey) {
    let children = self.children.remove(&key).unwrap_or_default();
    for child in children {
      self.gc_subtree(child);
    }
    self.elements.remove(&key);
    self.rects.remove(&key);
    self.signals.remove(&key);
    self.handlers.remove(&key);
    // Abort any tasks spawned by this element via `ctx.spawn`.
    if let Some(tasks) = self.tasks.remove(&key) {
      for (_, handle) in tasks {
        handle.abort();
      }
    }
    if let Some(subs) = self.subs_by_element.remove(&key) {
      for sig in subs {
        if let Some(set) = self.subs_by_signal.get_mut(&sig) {
          set.remove(&key);
        }
      }
    }
  }
}
```

**Unmount detection.** There is no global "is this element alive" mark.
Instead, when an element re-renders, the runtime snapshots its previous
child set and compares against the new child set populated by
`ctx.render` calls during the render. Children present before but not
after have been unmounted by their parent's logic (e.g. a conditional
that dropped them, or a loop that no longer includes their key), and
are GC'd recursively along with their entire subtrees. Ancestors and
siblings of the re-rendered element are untouched by this process —
as they should be, since they are still mounted.

### Event flow

1. `Event::Init` — mount the root element, render it, flush.
2. `Event::Key(k)` — broadcast to **every** registered `on_key` handler
   in the tree. Each handler decides whether the event is relevant to
   its element (e.g. by checking a `focused` prop or signal threaded
   down by the parent). Handlers may call `signal.set(..)` /
   `signal.update(..)`, which mutate storage immediately (see the
   Signal section) and post an `Event::SignalChanged(id)` notification
   onto the event channel.
3. `Event::Mouse(m)` — broadcast to every registered `on_mouse`
   handler. The handler is invoked with the mouse event *and* the
   element's current `Rectangle`, so it can hit-test against its own
   region (`rect.contains(m.pos)`).
4. `Event::SignalChanged(id)` — the runtime walks
   `subs_by_signal[id]` and adds every subscriber to `dirty`. The
   signal's storage was already mutated by the `Signal::set` /
   `Signal::update` call that produced this event; this step only
   handles dirty propagation. Because notifications arrive on the
   same channel as input events, the *order in which
   re-renders happen* is serialized even though storage writes are
   not.
5. After processing an event, drain `dirty`, re-rendering each
   element. A parent re-render re-invokes its child elements, so
   descendants are covered transitively. The drain is **sorted by
   ascending z** (see *Z-ordering and opacity* below) so a lower-z
   element is repainted before any higher-z overlay we promote in
   response.
6. Diff `back` against `front`, emit escape sequences, swap.

**Focus is not a runtime concept.** The runtime broadcasts; components
decide. The canonical pattern is for a parent to own a focus signal
(e.g. `Signal<MyFocusEnum>`) and pass the relevant slice of it to each
focusable child as a prop. The `Root` example below demonstrates this.

### Z-ordering and opacity

The runtime supports overlapping siblings (`ZStack`, `Modal`) without
requiring the *whole* affected subtree to re-render every time a leaf
below an overlay changes. Two pieces of per-element state make this
work:

- **`z: i32`** — the layering depth assigned to an element by its
  parent at placement time. The parent calls either `ctx.render(child,
  rect)` (child inherits the parent's z) or `ctx.render_with_z(child,
  rect, z)` (explicit override). `ZStack` is the canonical user of the
  latter: each i-th child is placed at `parent_z + 1 + i`, producing a
  strictly increasing z that matches placement order. `HStack` /
  `VStack` / `Centered` / `Border` use the inheriting form because
  their children don't overlap.

- **`opaque: bool`** — set during a render by `ctx.set_opaque()`. An
  element is opaque iff it fully paints every cell of its assigned
  rect this render. `Border`, `Input`, and other components that
  `region.fill(..)` their entire rect call this; pure-layout containers
  (`HStack`, `VStack`, `ZStack`, `Empty`) and components that only
  write a sub-pattern (`Text`, `List`) do not. The flag is reset at
  the start of each render and re-committed from the closure's calls,
  so an opacity-conditional component can toggle from frame to frame.

The partial-re-render protocol is:

1. `notify_signal_changed` adds the dirty leaf to `self.dirty` as today.
2. `render_dirty` drains in z-ascending order (stable sort by
   `nodes[k].z`).
3. For each dirty key `K`, `render_one(K, clear_rect=true)` clears
   `rects[K]`, re-runs `K`'s closure, and re-renders its placements
   synchronously in declaration order (so within `K`'s subtree, paint
   order matches `ctx.render(..)` order).
4. After `K` is repainted, `promote_overlays(K)` walks every mounted
   node `O` with `O != K`, `O.opaque`, `O.z > K.z`, and `rects[O]`
   intersects `rects[K]`, and inserts each such `O` into
   `self.dirty`. They are picked up by the next iteration of the
   outer `while !dirty.is_empty()` loop and rendered in z-ascending
   order, so each overlay paints over the cells it covers.

The rule for skipping transparent overlays is: their cells inside
`K`'s rect either don't belong to them (they didn't paint there) or
belong to one of their *descendants*, which has its own rect and is
considered independently by the overlap walk. A `ZStack` wrapper is
never opaque — but the opaque `Border` inside its overlay child is,
and that `Border` is what the walk picks up.

**Why z-sorted drain matters even without overlays.** If two
elements at different z's go dirty simultaneously (same event
produces signal mutations on both), processing them in `HashSet`
order could render the higher-z element first and then have the
lower-z element's `clear_rect` wipe part of it. Sorting by ascending
z is the cheap fix; overlap promotion then ensures the higher-z
element repaints on top after.

**Invariant the runtime relies on.** A correctly-authored opaque
component really does paint every cell of its rect. If a component
calls `set_opaque()` but leaves some cells untouched, those cells
will not be restored by `promote_overlays` (which simply re-runs the
overlay's render closure), and stale lower-z content will bleed
through. Components in the standard library (`Border`, `Input`)
uphold this by calling `region.fill(Cell::BLANK)` before drawing
their foreground.

## Buffer and Region

```rust
pub struct Cell {
  ch: char,
  fg: Color,
  bg: Color,
  attrs: Attributes,
}

pub struct Buffer {
  size: Size,
  cells: Vec<Cell>,   // row-major, len == size.w * size.h
}

impl Buffer {
  /// Resize and clear. New cells are blank; old contents are discarded.
  /// See "Resize handling" below for why this doesn't preserve overlap.
  pub fn resize(&mut self, size: Size);
  pub fn region(&mut self, rect: Rectangle) -> Region<'_>;
  pub fn full_rect(&self) -> Rectangle;
}

/// Scoped mutable view into a sub-rectangle of a Buffer.
pub struct Region<'a> {
  rect: Rectangle,
  buffer: &'a mut Buffer,
}

impl<'a> Region<'a> {
  pub fn rect(&self) -> Rectangle;
  pub fn set(&mut self, x: u16, y: u16, cell: Cell);
  pub fn write_str(&mut self, x: u16, y: u16, s: &str, style: Style);
  pub fn fill(&mut self, cell: Cell);
  pub fn sub_region(&mut self, rect: Rectangle) -> Region<'_>;
}
```

The `back` buffer is what components draw into. After a render pass,
`Runtime::flush` diffs `back` against `front` cell-by-cell and emits the
minimal escape sequence stream. Then `front` is updated to match.

**Resize handling.** `Buffer::resize` is destructive: it reallocates
blank cells at the new size and drops the old contents. On
`Event::Resize`, the runtime resizes only `back` and leaves `front`
stale at the old size. The next `Buffer::diff` then sees a size
mismatch and returns *every* cell of `back`, which `flush()` treats as
a clear-screen + full repaint. The freshly-rendered `back` (produced
by the root being marked dirty in the same event) is the sole source
of on-screen content after a resize, so preserving overlapping cells
in `Buffer::resize` would be dead work.

## Element

```rust
/// A runtime invocation of a component, ready to be rendered. Produced by
/// `ui!{ .. }` (which captures props and the call-site span into a
/// closure that calls the underlying component function).
///
/// When the runtime invokes an element with a Context, the component
/// body inside:
///   - calls hook constructors on `ctx` to look up its state slots,
///   - reads signals via `signal.get(ctx)`,
///   - registers event handlers via `ctx.on_key(..)`, etc.,
///   - registers child elements via `ctx.render(rect, child)` (which the
///     runtime will render into sub-rects of the current region),
///   - draws into `ctx.region()`.
pub type Element = Box<dyn FnMut(&mut Context)>;
```

## Context

A single `Context` type is passed to every render. It is the only API
surface a component author touches inside their function body.

```rust
pub struct Context<'a> {
  // Which element is currently being rendered. Used to attribute hooks,
  // subscriptions, child registrations, and handlers.
  current_key: ElementKey,
  // Region (sub-rect of the back buffer) this element is drawing into.
  region: Region<'a>,
  // Back-pointer to runtime tables. Concrete shape TBD; could be `&mut
  // Runtime` or a narrower struct of mutable borrows.
  rt: &'a mut RuntimeRef,
}

impl<'a> Context<'a> {
  /// Returns the region this element draws into.
  pub fn region(&mut self) -> &mut Region<'a>;

  /// Look up or create a signal slot at this call site.
  /// On first call: initializes with T::default() and returns a fresh handle.
  /// On subsequent renders: returns a handle to the existing slot.
  ///
  /// Storage lives in `Runtime.signals`; the returned `Signal<T>` is a
  /// cheap handle carrying a `SignalId` and an `EventSender`.
  #[track_caller]
  pub fn new_signal<T: Default + Send + 'static>(&mut self) -> Signal<T>;

  /// Like new_signal but with an explicit initializer (called only once).
  #[track_caller]
  pub fn new_signal_with<T: Send + 'static>(&mut self, init: impl FnOnce() -> T) -> Signal<T>;

  /// Register a key handler for the current element. Replaces any
  /// previous handler registered at this slot. The handler is invoked
  /// for *every* key event — the runtime does not track focus or
  /// route based on it. Handlers gate themselves (typically on a
  /// `focused` prop threaded down by the parent).
  #[track_caller]
  pub fn on_key(&mut self, f: impl Fn(KeyEvent) + 'static);

  /// Register a mouse handler. The handler receives the mouse event
  /// and the element's current `Rectangle`, so it can hit-test:
  /// `if rect.contains(event.pos) { .. }`. Like `on_key`, this fires
  /// for every mouse event regardless of position; the handler
  /// decides relevance.
  #[track_caller]
  pub fn on_mouse(&mut self, f: impl Fn(MouseEvent, Rectangle) + 'static);

  /// Register a child element to be rendered into a sub-rectangle of the
  /// current region. The macro `ui!{ .. }` produces calls to this.
  pub fn render(&mut self, rect: Rectangle, child: Element);

  /// Spawn a background task tied to the current element's lifetime.
  /// Like other hooks, this is keyed by call site (`#[track_caller]`):
  /// the task is spawned on the first render that reaches this call
  /// site for this element, and on subsequent renders the existing
  /// task is left running (the future passed in is dropped without
  /// being polled). The task is aborted when the element unmounts or
  /// the runtime is dropped.
  ///
  /// Tasks typically read state via `Signal::peek` and write via
  /// `Signal::set` / `Signal::update`. Both work from any thread
  /// because signals are `Send + Sync` shared storage.
  #[track_caller]
  pub fn spawn<F>(&mut self, fut: F)
  where
    F: Future<Output = ()> + Send + 'static;
}
```

## Signal

A `Signal<T>` is a cheap, `Clone + Send + Sync` handle wrapping shared
storage (`Arc<RwLock<T>>`). The runtime retains a type-erased `Arc` to
the same storage in `Runtime.signals` so it outlives the component
body's local handles and can be dropped at unmount.

This design is structurally similar to SolidJS's signal model:
storage is shared, reads are cheap and direct, writes notify
subscribers. The differences are Rust-specific:
- subscriptions require an explicit `&mut Context` (not a thread-local
  reactive-context stack), and
- write notifications go through the runtime's event channel so
  re-render scheduling stays serialized with input events.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SignalId {
  element: ElementKey,
  slot:    SlotId,
}

pub struct Signal<T> {
  id: SignalId,
  cell: Arc<RwLock<T>>,
  tx: EventSender,
}

impl<T> Clone for Signal<T> { /* clones id, Arc, tx */ }

impl<T: Send + Sync + 'static> Signal<T> {
  /// Read the current value AND subscribe the currently-rendering
  /// element to changes. Only valid during render; the `&mut Context`
  /// argument is what makes this so.
  pub fn get(&self, ctx: &mut Context) -> T where T: Clone;

  /// Read the current value without subscribing. Safe to call from
  /// anywhere: event handlers, spawned tasks, other threads.
  pub fn peek(&self) -> T where T: Clone;

  /// Replace the value and post a `SignalChanged(id)` notification
  /// onto the event channel. The store happens immediately (under the
  /// lock); the dirty-marking and re-rendering happen when the event
  /// loop processes the notification.
  pub fn set(&self, value: T);

  /// Read-modify-write under the lock. The closure receives `&mut T`
  /// and is run synchronously. After it returns, a
  /// `SignalChanged(id)` notification is posted.
  pub fn update<F>(&self, f: F)
  where
    F: FnOnce(&mut T);
}
```

**`peek` is the read primitive for non-render code.** Event handlers,
spawned tasks, and any other code that wants the current value but
must not (or cannot) subscribe should call `peek`. Inside a render
body, prefer `get` so the element correctly re-renders when the
signal changes.

**Mutations are synchronous; notifications are async.** `set` and
`update` complete the storage write before returning. The dirty-mark
and re-render happen on the next event-loop tick, when
`Event::SignalChanged(id)` is processed. Within a single handler,
back-to-back `set` calls are immediately visible to subsequent
`peek`s in the same handler, which is the natural model for
read-modify-write code paths in spawned tasks.

**Subscription lifecycle:**
- `Signal::get(ctx)` adds `ctx.current_key` to `subs_by_signal[self.id]`
  and `self.id` to `subs_by_element[ctx.current_key]`. The `HashSet`
  in `subs_by_signal` makes repeated `get`s of the same signal
  idempotent.
- Subscriptions are **not** cleared between renders. By the
  Subscription stability invariant (see Identity section), each
  element's subscription set is stable across all of its renders.
- When an element unmounts, `gc_subtree` walks `subs_by_element[K]`,
  removes `K` from each referenced signal's subscriber set, and drops
  the entry.

## End-to-end example

```rust
#[derive(Default, Clone)]
pub enum Focus {
  #[default]
  Tasks,
  Timer,
}

impl Focus {
  fn toggle(self) -> Self {
    match self {
      Self::Tasks => Self::Timer,
      Self::Timer => Self::Tasks,
    }
  }
}

#[component]
pub fn Root(ctx: &mut Context) {
  let focus = ctx.new_signal::<Focus>();

  ctx.on_key({
    let focus = focus.clone();
    move |key| {
      if key.code == KeyCode::Esc {
        focus.update(|f| *f = f.toggle());   // mutates under lock, posts notification
      }
    }
  });

  ui! {
    HStack(flex: [Frac(1), Frac(1)]) {
      Section(title: "Tasks", focused: matches!(focus.get(ctx), Focus::Tasks)) {
        Text(text: "some tasks here...")
      }
      Section(title: "Timer", focused: matches!(focus.get(ctx), Focus::Timer)) {
        Timer()
      }
    }
  }
}

#[component]
pub fn Timer(ctx: &mut Context) {
  let elapsed = ctx.new_signal::<u64>();
  ui! {
    Text(text: format!("{}s", elapsed.get(ctx)))
  }
}
```

Trace of an Esc keypress, given a mounted tree of elements `Root → HStack
→ {Section, Section}`:

1. `Event::Key(Esc)` arrives.
2. Runtime broadcasts the key to every `on_key` handler. The Root
   element's handler matches and calls `focus.update(|f| *f = f.toggle())`,
   which acquires the lock on `focus`'s storage, mutates it in-place,
   and posts `Event::SignalChanged(focus_id)` onto the event channel.
   Returns immediately. Storage is now toggled.
3. The runtime drains `dirty` (empty) and flushes (no-op), then loops
   back to the next event.
4. `Event::SignalChanged(focus_id)` arrives. The runtime walks
   `subs_by_signal[focus_id]`, finds Root, and inserts it into
   `dirty`.
5. `render_dirty` runs: re-renders Root, which re-invokes its closure.
   `focus.get(ctx)` reads the (already-toggled) value. `ui!{}`
   re-registers HStack and its two Sections; HStack assigns rects and
   the runtime renders each child into them. Descendants re-render
   transitively.
6. `flush()` diffs `back` vs `front`, emits the minimal escape sequence
   stream (the two Section title-bar styles flipped, nothing else).

## Implementation order

1. `Buffer` + `Region` + diff + flush. Verifiable via a non-interactive
   test that draws fixed cells and compares emitted bytes.
2. `Runtime` event loop + `mount` + `render` for hand-written elements
   (raw `Box<dyn FnMut(&mut Context)>` closures). No signals yet — render
   once on Init, exit on Esc.
3. `Context::new_signal`, `Signal<T>`, dirty queue, subscription tables,
   `render_dirty`. Demo: a counter component with `+`/`-` keys, still
   hand-written elements.
4. `ctx.render(rect, child)`, child layout helpers (`HStack`, `VStack`).
   Still hand-written elements.
5. `#[component]` and `ui!{ .. }` macros. Sugar over the above — they
   turn component definitions and call sites into the closures the
   runtime consumes.

## Open questions

- **Unused-slot policy.** Drop silently (lenient) vs panic (strict, like
  React). Leaning lenient since `#[track_caller]` already guarantees
  stable identity, so unused slots can only mean an `if`-gated hook,
  which is fine. (See the `MyComponent` example in the design discussion
  where state from one branch is legitimately discarded when the other
  branch is taken.)
- **Subscription-stability enforcement.** Debug-only check that
  `(ElementKey, get_call_site) → SignalId` is constant across renders,
  panicking on mismatch. Cheap; worth doing in debug builds at minimum.
- **Effects.** Not yet specified. Probably `ctx.on_mount(f)`, `ctx.effect(f,
  deps)` analogous to `useEffect`. Defer to v2.
- **Async / multi-thread.** Signals are `Send + Sync` (storage is
  `Arc<RwLock<T>>`), so mutations from spawned tasks or other threads
  work natively. The `Event::SignalChanged` notification crosses the
  channel back to the runtime thread for re-render scheduling.
- **Drop semantics.** Unmount cleanup is handled by `gc_subtree` (see
  Runtime), driven by per-element child-set diffing on re-render.

## Notes from v1 implementation

- **Child elements must be passed as factories, not as values.** Because
  `ctx.render(rect, child)` consumes its `Element` argument and a
  parent's render closure runs many times, components that contain
  children take a *factory* `FnMut() -> Element` (or `FnMut() ->
  Vec<Element>` for n-ary containers). The factory produces a fresh
  `Element` per render that closes over the parent's latest captured
  state. This isn't visible in the public API of
  `Context::render` itself, but it shapes how every container
  component is written. The macros (`#[component]`, `ui!{}`) will
  hide this by emitting factories from declarative syntax.
