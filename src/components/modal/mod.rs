mod hook;

pub use self::hook::{use_modal_funcs, Funcs};
use crate::{
  components::{
    children::Children,
    element::{Any as AnyElement, Element},
    Component,
  },
  event::KeyEvent,
  state::use_state,
  terminal::{Frame, Rect},
};

#[derive(Default)]
pub struct Modal {
  pub children: Children<1>,
}

impl Component for Modal {
  fn render(&self) -> AnyElement {
    let modal = use_state(|| None);
    let funcs = use_state(|| Funcs::new(modal.clone()));

    hook::set_modal_funcs(funcs.get());

    AnyElement::new(Frozen {
      content: self.children[0].render(),
      modal: modal.get().map(|modal| modal.render()),
    })
  }
}

struct Frozen {
  content: AnyElement,
  modal: Option<AnyElement>,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.content.on_key(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.content.draw(rect, frame);

    if let Some(modal) = &self.modal {
      modal.draw(rect, frame);
    }
  }
}
