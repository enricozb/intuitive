use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
  text::{Lines, Spans},
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
      lines: self.text.clone().into(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen {
  lines: Lines,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new::<Vec<TuiSpans>>(self.lines.0.iter().cloned().map(TuiSpans::from).collect());

    frame.render_widget(widget, rect);
  }
}
