use tui::{
  style::{Color, Style},
  widgets::{Block, Borders},
};

use crate::{
  components::{children::Children, Component},
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Section {
  pub title: String,
  pub color: Option<Color>,

  pub children: Children<1>,
  pub on_key: KeyHandler,
}

impl Component for Section {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      title: self.title.clone(),
      color: self.color.unwrap_or(Color::White),

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
