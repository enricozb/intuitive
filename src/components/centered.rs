use crate::{
  components::{children::Children, AnyComponent, Component, Embed, Empty, HStack, VStack},
  render,
  terminal::{Frame, Rect},
};

#[derive(Clone, Default)]
pub struct Centered {
  pub children: Children<1>,
}

impl Component for Centered {
  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.render().draw(rect, frame);
  }

  fn render(&self) -> AnyComponent {
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
