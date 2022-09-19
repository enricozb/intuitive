use std::sync::Arc;

use parking_lot::Mutex;

use crate::{components::Any as AnyComponent, state::State};

static FUNCS: Mutex<Option<Funcs>> = Mutex::new(None);

/// A structure returned by [`use_modal`] that controls the hiding/showing of a modal.
///
/// [`use_modal`]: fn.use_modal.html
#[derive(Clone)]
pub struct Funcs {
  modal: State<Option<AnyComponent>>,

  show: Arc<dyn Fn(AnyComponent) + Send + Sync>,
  hide: Arc<dyn Fn() + Send + Sync>,
}

impl Funcs {
  pub(crate) fn new(modal: State<Option<AnyComponent>>) -> Self {
    let show_modal = modal.clone();
    let hide_modal = modal.clone();

    Self {
      modal,
      show: Arc::new(move |component| show_modal.set(Some(component))),
      hide: Arc::new(move || hide_modal.set(None)),
    }
  }

  /// Return whether if the modal is shown.
  pub fn is_shown(&self) -> bool {
    self.modal.inspect(Option::is_some)
  }

  /// Set `component` to be shown by the modal.
  pub fn show(&self, component: AnyComponent) {
    (self.show)(component);
  }

  /// Hide the showing modal, if any.
  pub fn hide(&self) {
    (self.hide)();
  }
}

/// A hook that can control the hiding/showing of a modal.
///
/// Like [`use_state`], calls to `use_modal` may only be within a call to
/// [`Component::render`]. Unlike [`use_state`], calls to `use_modal` may only be within
/// a component that is a child component of some [`Modal`]. The [`Funcs`] returned by
/// `use_modal` will then refer to the nearest ancestor [`Modal`]. For example, if we
/// have the following layout:
/// ```rust
/// # use intuitive::{render, component, components::{Empty, experimental::modal::{use_modal, Modal}}};
/// #
/// #[component(MyComponent)]
/// fn render() {
///   let modal = use_modal();
///
///   render! {
///     Empty()
///   }
/// }
///
/// #[component(Root)]
/// fn render() {
///   render! {
///     Modal() {     // modal 1
///       Modal() {   // modal 2
///         Modal() { // modal 3
///           MyComponent()
///         }
///       }
///     }
///   }
/// }
/// ```
/// and `use_modal` is called within `MyComponent`, then it will return a [`Funcs`] struct
/// that acts on `modal 3`. The other two ancestor modals are inaccessible.
///
/// [`Component::render`]: trait.Component.html#tymethod.render
/// [`Modal`]: struct.Modal.html
/// [`Funcs`]: struct.Funcs.html
/// [`use_state`]: ../../state/fn.use_state.html
pub fn use_modal() -> Funcs {
  FUNCS
    .lock()
    .clone()
    .expect("use modal called outside of a Modal or outside of render")
}

pub fn set_modal_funcs(funcs: Funcs) {
  *FUNCS.lock() = Some(funcs);
}
