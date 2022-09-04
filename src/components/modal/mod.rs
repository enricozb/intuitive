mod hook;

pub use self::hook::{use_modal_funcs, Funcs};
use crate::{
  components::{children::Children, AnyComponent, Component},
  event::KeyEvent,
  state::{use_state, State},
  terminal::{Frame, Rect},
};

#[derive(Default)]
pub struct Modal {
  pub children: Children<1>,
  pub modal: State<Option<AnyComponent>>,
}

impl Component for Modal {
  fn render(&self) -> AnyComponent {
    let funcs = use_state(|| Funcs::new(self.modal.clone()));
    hook::set_modal_funcs(funcs.get());

    Frozen {
      content: self.children[0].render(),
      modal: self.modal.get().map(|modal| modal.render()),
    }
    .into()
  }
}

struct Frozen {
  content: AnyComponent,
  modal: Option<AnyComponent>,
}

impl Component for Frozen {
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
