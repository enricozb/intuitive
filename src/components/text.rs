use tui::widgets::Paragraph;

use crate::{
  components::{AnyComponent, Component},
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Text {
  pub text: String,

  pub on_key: KeyHandler,
}

impl Component for Text {
  fn on_key(&self, event: KeyEvent) {
    (self.on_key)(event)
  }

  fn render(&self) -> AnyComponent {
    self.clone().into()
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new(self.text.clone());

    frame.render_widget(widget, rect);
  }
}
