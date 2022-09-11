use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  spans::{Span, Spans},
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
      text: self.text.clone().into(),
      on_key: self.on_key.clone(),
    })
  }
}

#[derive(Clone)]
struct Lines(Vec<Spans>);

impl From<Spans> for Lines {
  fn from(spans: Spans) -> Self {
    let mut expanded = Vec::new();

    for span in spans {
      let lines: Vec<&str> = span.text.split('\n').collect();
      expanded.push(Some(Span::new(lines[0], span.style)));

      for line in &lines[1..] {
        expanded.push(None);
        expanded.push(Some(Span::new(*line, span.style)));
      }
    }

    let split = expanded
      .split(|span| span.is_none())
      .map(|spans| Spans::new(spans.into_iter().flatten().cloned().collect::<Vec<Span>>()))
      .collect();

    Lines(split)
  }
}

struct Frozen {
  text: Lines,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let widget = Paragraph::new::<Vec<TuiSpans>>(self.text.clone().0.into_iter().map(TuiSpans::from).collect());

    frame.render_widget(widget, rect);
  }
}
