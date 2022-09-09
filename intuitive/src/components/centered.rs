use crate::{
  components::{children::Children, Component, Embed, Empty, HStack, VStack},
  element::Any as AnyElement,
  render,
};

/// A component for centering its contents.
///
/// For example,
/// ```rust
/// render! {
///   Centered() {
///     Section(title: "I'm centered")
///   }
/// }
/// ```
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
          Embed(content: self.children[0].clone())
          Empty()
        }
        Empty()
      }
    }
  }
}
