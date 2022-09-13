use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler, MouseEvent, MouseHandler},
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
  pub on_mouse: MouseHandler,
}

impl Component for Text {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      lines: self.text.clone().into(),
      on_key: self.on_key.clone(),
      on_mouse: self.on_mouse.clone(),
    })
  }
}

struct Frozen {
  lines: Lines,
  on_key: KeyHandler,
  on_mouse: MouseHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn on_mouse(&self, _rect: Rect, event: MouseEvent) {
    self.on_mouse.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new::<Vec<TuiSpans>>(self.lines.0.iter().cloned().map(TuiSpans::from).collect());

    frame.render_widget(widget, rect);
  }
}
