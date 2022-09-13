//! A module containing the `Scroll` component.

use std::cmp;

use tui::{
  buffer::Buffer,
  text::Spans as TuiSpans,
  widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  state::use_state,
  style::Style,
  terminal::{Frame, Rect},
  text::{Lines, Spans},
};

/// A component that displays text along with a scrollbar.
///
/// `Scroll` renders the `Spans` passed into it along with a scrollbar.
///
/// [`Section`]: ../../struct.Section.html
#[derive(Default)]
pub struct Scroll {
  pub title: Spans,
  pub border: Style,
  pub text: Spans,

  pub on_key: KeyHandler,
}

impl Component for Scroll {
  fn render(&self) -> AnyElement {
    let offset = use_state(|| 0);

    AnyElement::new(Frozen {
      title: self.title.clone(),
      border: self.border,
      lines: self.text.clone().into(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen {
  title: Spans,
  lines: Lines,
  border: Style,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event)
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    frame.render_widget(self, rect);
  }
}

impl Widget for &Frozen {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    let block = Block::default()
      .title::<TuiSpans>((&self.title).into())
      .borders(Borders::ALL)
      .border_style(self.border.into());

    // render text
    let paragraph = Paragraph::new::<Vec<TuiSpans>>(self.lines.0.iter().cloned().map(TuiSpans::from).collect()).block(block);
    Widget::render(paragraph, rect, buf);

    // render scroll bar
    let num_lines = self.lines.0.len();
    let height = (rect.height - 2) as usize;
    let scroll_height = cmp::min(height, height * height / num_lines) as u16;
    let offset = 5;

    eprintln!("scroll height: {}", scroll_height);

    buf.set_string(rect.right() - 1, rect.top(), "▲", self.border.into());
    buf.set_string(rect.right() - 1, rect.bottom() - 1, "▼", self.border.into());

    for y in 1..=scroll_height {
      buf.set_string(rect.x + rect.width - 1, rect.y + y + offset, "█", self.border.into());
    }
  }
}
