use crate::{
  component,
  components::{children::Children, Embed, Empty, HStack, VStack},
  event::KeyHandler,
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
#[component(crate::Centered)]
pub fn render(children: Children<1>, on_key: KeyHandler) -> AnyElement {
  render! {
    VStack(on_key) {
      Empty()
      HStack() {
        Empty()
        Embed(content: children[0].clone())
        Empty()
      }
      Empty()
    }
  }
}
