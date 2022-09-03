use crate::{
  components::{AnyComponent, Component},
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn draw(&self, _rect: Rect, _frame: &mut Frame) {}

  fn render(&self) -> AnyComponent {
    self.clone().into()
  }
}
