use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  spans::Spans,
  terminal::{Frame, Rect},
};

/// A component that displays text.
///
/// `Text` renders the `Spans` passed into it.
#[derive(Default)]
pub struct Text {
  pub text: Spans,

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
  text: Spans,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new::<TuiSpans>((&self.text).into());

    frame.render_widget(widget, rect);
  }
}
