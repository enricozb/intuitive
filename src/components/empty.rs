use crate::components::{AnyComponent, Component};

#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self) -> AnyComponent {
    self.clone().into()
  }
}
