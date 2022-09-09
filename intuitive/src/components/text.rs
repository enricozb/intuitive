use tui::widgets::Paragraph;

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
};

/// A component to display text.
///
/// `Text` renders the `String` passed into it.
#[derive(Default)]
pub struct Text {
  pub text: String,

  pub on_key: KeyHandler,
}

impl Component for Text {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      text: self.text.clone(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen {
  text: String,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new(self.text.clone());

    frame.render_widget(widget, rect);
  }
}
