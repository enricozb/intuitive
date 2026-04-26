//! The `Element` value type — what `ui!{}` returns and what users pass
//! around.
//!
//! An `Element` is a *pure data* tree node, not a runtime registration.
//! Constructing one is cheap and has no side effects. The runtime mounts
//! `Element`s into per-key `Node`s (see `runtime::Node`) when it walks
//! them during a render. Until then, an `Element` is just a value: a
//! render closure, a span identifying its source location, an optional
//! user-supplied key, and a `Vec<Element>` of composed children.
//!
//! ```ignore
//! let nested: Element = ui! { Text(text: "hello") };
//! let outer:  Element = ui! { Border() { #nested.clone() } };   // future syntax
//! ```
//!
//! Today there is no splice syntax (`#expr`) in `ui!{}`; children inside
//! a `{ ... }` block must be component invocations or control flow
//! producing component invocations. `Element` itself is `Clone`, so
//! once such syntax is added, splicing the same Element value into
//! multiple parents will Just Work.
//!
//! ## `Clone` and `Send`
//!
//! `Element` is `Clone`: cloning is a cheap `Arc` bump for the render
//! closure plus deep `Clone`s of the identity metadata and composed
//! children (which are themselves Element clones, so likewise cheap).
//! Each clone, when mounted under a distinct `ElementKey`, is an
//! independent instance: per-key signals/handlers/tasks are
//! independent, even though all instances share the *same* underlying
//! closure (so they draw the same way and observe the same captured
//! props).
//!
//! `Element` is `Send` (so it can move between threads, e.g. through
//! a `tokio::sync::mpsc` channel) but not `Sync` (its render closure
//! is locked through a `Mutex` for invocation; sharing `&Element`
//! across threads has no use case in this runtime).
//!
//! See `design.md` for the rendering protocol and `macros.md` for the
//! `ui!{}` lowering.

use std::sync::{Arc, Mutex};

use crate::runtime::{Context, Span};

/// The closure that an element runs each time it is rendered.
///
/// Wrapped in `Arc<Mutex<...>>` so `Element` can be `Clone` and
/// `Send`: cloning bumps the Arc; invoking the closure briefly
/// locks the Mutex. The inner trait object is `FnMut + Send +
/// 'static`. `Send` lets `Mutex<Box<...>>` itself be `Send`, which
/// is what propagates to `Element: Send`.
///
/// Re-entrant invocation of the same closure (i.e. a closure that
/// somehow ends up rendering an Element that aliases itself) will
/// deadlock on the Mutex. Macro-generated closures never do this;
/// hand-rolled closures should avoid it.
pub type RenderFn = Arc<Mutex<Box<dyn FnMut(&mut Context<'_>) + Send>>>;

/// A node in the user-facing tree. Constructed by `#[component]`
/// `build()` methods and stamped with identity metadata by `ui!{}`.
///
/// Field visibility is `pub(crate)` so the macros and runtime can fill
/// these in directly; user code constructs `Element`s through the
/// builder chain emitted by `ui!{}` (or via `Element::new` + setters
/// for hand-rolled cases like tests).
#[derive(Clone)]
pub struct Element {
  /// Source-location identity for this element. Stamped by `ui!{}`
  /// at each child call site so that, e.g., two `Text(...)` children
  /// on different lines have distinct spans (and hence distinct
  /// `ElementKey`s once mounted).
  pub(crate) span: Span,

  /// User-supplied key, set via `key:` at the call site. Combined
  /// with `span` plus the mount-parent's key to form an `ElementKey`.
  pub(crate) user_key: Option<String>,

  /// Children composed in at `ui!{}` time (the `{ ... }` block).
  ///
  /// Stored as a plain `Vec<Element>`: each entry is the cheap-Clone
  /// snapshot the parent supplied. The runtime clones an entry (Arc
  /// bump on the render closure) when it needs to mount it as a
  /// child Node, leaving the original in place for re-mounting later
  /// (e.g. when a `Select`-style parent toggles which child is
  /// active).
  pub(crate) children: Vec<Element>,

  /// The element's render behavior — what it does each time the
  /// runtime asks it to render. Shared via `Arc<Mutex<>>` so the
  /// Element can be cloned and shipped between threads.
  pub(crate) render: RenderFn,
}

impl Element {
  /// Build an `Element` with no identity metadata and no composed
  /// children. Setters (`with_span`, `with_user_key`, `with_children`)
  /// fill in the rest; `ui!{}` emits exactly that chain.
  ///
  /// Component-generated `build()` methods call this and then return
  /// the result; `ui!{}` stamps identity metadata onto it at the
  /// call site.
  #[must_use]
  pub fn new<F>(render: F) -> Self
  where
    F: FnMut(&mut Context<'_>) + Send + 'static,
  {
    Self {
      span: Span::placeholder(),
      user_key: None,
      children: Vec::new(),
      render: Arc::new(Mutex::new(Box::new(render))),
    }
  }

  /// Stamp this element's source-location span. Called by `ui!{}`
  /// at each child call site so siblings on different source lines
  /// end up with distinct spans.
  #[must_use]
  pub fn with_span(mut self, span: Span) -> Self {
    self.span = span;
    self
  }

  /// Stamp this element's user-supplied key (from `key: <expr>` at
  /// the call site). Disambiguates siblings sharing a span (e.g.
  /// elements produced inside a `for` loop in `ui!{}`).
  #[must_use]
  pub fn with_user_key(mut self, key: Option<String>) -> Self {
    self.user_key = key;
    self
  }

  /// Replace the composed-children vec. Called by `ui!{}` after
  /// gathering the `{ ... }` block's contents into a `Vec<Element>`.
  #[must_use]
  pub fn with_children(mut self, children: Vec<Element>) -> Self {
    self.children = children;
    self
  }

  /// The element's source-location span.
  #[must_use]
  pub fn span(&self) -> Span {
    self.span
  }

  /// The element's user-supplied key, if any.
  #[must_use]
  pub fn user_key(&self) -> Option<&str> {
    self.user_key.as_deref()
  }
}
