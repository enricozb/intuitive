use crate::{
  components::{children::Children, element::Any as AnyElement, Component, Embed, Empty, HStack, VStack},
  render,
};

#[derive(Clone, Default)]
pub struct Centered {
  pub children: Children<1>,
}

impl Component for Centered {
  fn render(&self) -> AnyElement {
    render! {
      VStack() {
        Empty()
        HStack() {
          Empty()
          Embed(component: self.children[0].clone())
          Empty()
        }
        Empty()
      }
    }
  }
}
