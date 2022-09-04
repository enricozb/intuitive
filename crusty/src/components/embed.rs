use crate::{
  components::{AnyComponent, Component},
  element::Any as AnyElement,
};

#[derive(Default)]
pub struct Embed {
  pub component: AnyComponent,
}

impl Component for Embed {
  fn render(&self) -> AnyElement {
    self.component.render()
  }
}
