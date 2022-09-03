use tui::layout::{Constraint, Direction, Layout};

use super::flex::{Array as FlexArray, Flex};
use crate::{
  components::{children::Children, AnyComponent, Component},
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Stack<const N: usize> {
  pub flex: FlexArray<N>,
  pub children: Children<N>,
}

impl<const N: usize> Stack<N> {
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

    let grow_px = rect.width - total_px;

    Layout::default()
      .direction(Direction::Horizontal)
      .constraints(self.flex.map(|flex| match flex {
        Flex::Block(px) => Constraint::Length(px),
        Flex::Grow(grow) => Constraint::Length(grow * grow_px / total_grow),
      }))
      .split(rect)
  }
}

impl<const N: usize> Component for Stack<N> {
  fn render(&self) -> AnyComponent {
    self.clone().into()
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let layout = self.layout(rect);

    for (i, child) in self.children.iter().enumerate() {
      child.draw(*layout.get(i).expect("missing rect"), frame);
    }
  }
}
