use crate::components::{AnyComponent, Component};

#[derive(Default)]
pub struct Embed {
  pub component: AnyComponent,
}

impl Component for Embed {
  fn render(&self) -> AnyComponent {
    self.component.clone()
  }
}
