use tui::layout::{Constraint, Direction, Layout};

use super::{Flex, FlexArray};
use crate::{
  components::{children::Children, Component},
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Stack<const N: usize> {
  pub flex: FlexArray<N>,

  pub children: Children<N>,
  pub on_key: KeyHandler,
}

impl<const N: usize> Component for Stack<N> {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      flex: self.flex,

      children: self.children.render(),
      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen<const N: usize> {
  flex: FlexArray<N>,

  children: [AnyElement; N],
  on_key: KeyHandler,
}

impl<const N: usize> Frozen<N> {
  fn layout(&self, rect: Rect) -> Vec<Rect> {
    let total_grow: u16 = self
      .flex
      .iter()
      .map(|flex| match flex {
        Flex::Grow(grow) => *grow,
        Flex::Block(_) => 0,
      })
      .sum();

    let total_px: u16 = self
      .flex
      .iter()
      .map(|flex| match flex {
        Flex::Block(px) => *px,
        Flex::Grow(_) => 0,
      })
      .sum();

    let grow_px = rect.height - total_px;

    Layout::default()
      .direction(Direction::Vertical)
      .constraints(self.flex.map(|flex| match flex {
        Flex::Block(px) => Constraint::Length(px),
        Flex::Grow(grow) => Constraint::Length(grow * grow_px / total_grow),
      }))
      .split(rect)
  }
}

impl<const N: usize> Element for Frozen<N> {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle(event);
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let layout = self.layout(rect);

    for (i, child) in self.children.iter().enumerate() {
      child.draw(*layout.get(i).expect("missing rect"), frame);
    }
  }
}
