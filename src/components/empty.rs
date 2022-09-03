use crate::components::{Any as AnyComponent, Component};

#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self) -> AnyComponent {
    Clone::clone(self).into()
  }
}
