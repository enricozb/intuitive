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
  event::{KeyEvent, KeyHandler, MouseEvent, MouseEventKind},
  state::{use_state, State},
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

      offset,
    })
  }
}

struct Frozen {
  title: Spans,
  lines: Lines,
  border: Style,
  on_key: KeyHandler,

  offset: State<u16>,
}

impl Frozen {
  fn scroll_height(&self, rect: Rect) -> u16 {
    let num_lines = self.lines.0.len();
    let height = (rect.height - 2) as usize;

    cmp::min(height, height * height / num_lines) as u16
  }
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event)
  }

  fn on_mouse(&self, rect: Rect, event: MouseEvent) {
    let scroll_height = self.scroll_height(rect);
    let max_offset = rect.height - scroll_height - 2;

    match event.kind {
      MouseEventKind::ScrollDown => self.offset.update(|offset| cmp::min(max_offset, offset + 1)),
      MouseEventKind::ScrollUp => self.offset.update(|offset| offset.saturating_sub(1)),

      _ => (),
    }
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

    let offset = self.offset.get();

    // render text
    let paragraph =
      Paragraph::new::<Vec<TuiSpans>>(self.lines.0.iter().skip(offset as usize).cloned().map(TuiSpans::from).collect()).block(block);

    Widget::render(paragraph, rect, buf);

    buf.set_string(rect.right() - 1, rect.top(), "▲", self.border.into());
    buf.set_string(rect.right() - 1, rect.bottom() - 1, "▼", self.border.into());

    for y in 1..=self.scroll_height(rect) {
      buf.set_string(rect.x + rect.width - 1, rect.y + y + offset, "█", self.border.into());
    }
  }
}
