use crate::components::{element::Any as AnyElement, AnyComponent, Component};

#[derive(Default)]
pub struct Embed {
  pub component: AnyComponent,
}

impl Component for Embed {
  fn render(&self) -> AnyElement {
    self.component.render()
  }
}
