use tui::{
  style::{Color, Style},
  widgets::{Block, Borders},
};

use crate::{
  components::{
    children::Children,
    element::{Any as AnyElement, Element},
    Component,
  },
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
};

#[derive(Clone)]
pub struct Section {
  pub title: String,
  pub color: Color,

  pub children: Children<1>,
  pub on_key: KeyHandler,
}

impl Default for Section {
  fn default() -> Self {
    Self {
      title: String::default(),
      color: Color::White,

      children: Children::default(),
      on_key: KeyHandler::default(),
    }
  }
}

impl Component for Section {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      title: self.title.clone(),
      color: self.color,

      content: self.children[0].render(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen {
  title: String,
  color: Color,

  content: AnyElement,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle_or(event, |event| self.content.on_key(event));
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let block = Block::default()
      .title(self.title.as_ref())
      .borders(Borders::ALL)
      .border_style(Style::default().fg(self.color));

    self.content.draw(block.inner(rect), frame);
    frame.render_widget(block, rect);
  }
}
