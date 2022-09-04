mod hook;

pub use self::hook::{use_modal_funcs, Funcs};
use crate::{
  components::{children::Children, Component},
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  state::use_state,
  terminal::{Frame, Rect},
};

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
