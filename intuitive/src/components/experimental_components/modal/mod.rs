//! A module containing the `Modal` component and related hooks.
//!
//! The [`Modal`] component allows a user to present a [`Component`] on top of
//! the children of the modal, from anywhere inside it. This is useful for popups such
//! as input boxes, or error messages. See the [`Modal`] structure ocumentation for details.
//!
//! [`Modal`]: struct.Modal.html

mod hook;

pub use self::hook::{use_modal, Funcs};
use crate::{
  components::{children::Children, Component},
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  state::use_state,
  terminal::{Frame, Rect},
};

/// A component supporting modal-style overlays.
///
/// The [`Modal`] component allows a user to present a [`Component`] on top of
/// the children of the modal, from anywhere inside it. This is useful for popups such
/// as input boxes, or error messages.
///
/// For example, if we wanted to show an error message
/// whenever the `Enter` key is pressed, we can do something like this:
/// ```rust
/// # use intuitive::{component, components::{Centered, Section, Text, experimental::modal::{use_modal, Modal}}, style::Color, render, on_key};
/// #
/// #[component(MyComponent)]
/// fn render() {
///   let modal = use_modal();
///
///   let on_key = on_key! {
///     KeyEvent { code: Enter, .. } => modal.show(render! {
///       Centered() {
///         Section(title: "Error", border: Color::Red) {
///           Text(text: "Enter was pressed!")
///         }
///       }
///     }),
///
///     KeyEvent { code: Esc, .. } if modal.is_shown() => modal.hide(),
///     KeyEvent { code: Esc, .. } => event::quit(),
///   };
///
///   render! {
///     Section(title: "Some Example UI", on_key)
///   }
/// }
///
/// #[component(Root)]
/// fn render() {
///   render! {
///     Modal() {
///       MyComponent()
///     }
///   }
/// }
///
/// ```
/// In order to overlay an error message on top of `MyComponent`, we render it
/// as a child of a [`Modal`]. Then, in any descendant of this [`Modal`], we can call
/// [`use_modal`] to mutate the state of that [`Modal`].
///
/// # Internals
/// The [`Modal`] is somewhat special in that it does not (yet) use the built-in
/// [`use_state`] hooks, but instead has its own internal `static` hook manager.
///
/// [`Modal`]: struct.Modal.html
/// [`Component`]: trait.Component.html
/// [`use_state`]: ../../state/fn.use_state.html
/// [`use_modal`]: fn.use_modal.html
#[derive(Default)]
pub struct Modal {
  pub children: Children<1>,
  pub on_key: KeyHandler,
}

impl Component for Modal {
  fn render(&self) -> AnyElement {
    let modal = use_state(|| None);
    let funcs = use_state(|| Funcs::new(modal.clone()));

    hook::set_modal_funcs(funcs.get());

    AnyElement::new(Frozen {
      modal: modal.get().map(|modal| modal.render()),

      content: self.children[0].render(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen {
  modal: Option<AnyElement>,

  content: AnyElement,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle_or(event, |event| self.content.on_key(event));
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.content.draw(rect, frame);

    if let Some(modal) = &self.modal {
      modal.draw(rect, frame);
    }
  }
}
