use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler, MouseEvent, MouseHandler},
  terminal::{Frame, Rect},
  text::Lines,
};

/// A component that displays text.
///
/// `Text` renders the [`Lines`] passed into it.
///
/// [`Lines`]: ../text/struct.Lines.html
#[component(Text)]
pub fn render(text: Lines, on_key: KeyHandler, on_mouse: MouseHandler) {
  AnyElement::new(Frozen {
    lines: text.clone(),
    on_key: on_key.clone(),
    on_mouse: on_mouse.clone(),
  })
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
