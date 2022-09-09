use crate::{
  components::{Any as AnyComponent, Component},
  element::Any as AnyElement,
};

pub enum Content {
  Component(AnyComponent),
  Element(AnyElement),
}

impl From<AnyComponent> for Content {
  fn from(component: AnyComponent) -> Self {
    Self::Component(component)
  }
}

impl From<AnyElement> for Content {
  fn from(element: AnyElement) -> Self {
    Self::Element(element)
  }
}

impl Default for Content {
  fn default() -> Self {
    Self::Element(AnyElement::default())
  }
}

/// A component renders an [`element::Any`] or a [`component::Any`].
///
/// This is often needed when rendering children. More generally, `Embed` is
/// useful when you have a variable that contains an [`element::Any`] or
/// a [`component::Any`] and you want to insert it into a [`render!`] call.
///
/// For example,
/// ```rust
/// #[component(Centered)]
/// pub fn render(children: Children<1>) -> AnyElement {
///   render! {
///     VStack() {
///       Empty()
///       HStack() {
///         Empty()
///         Embed(content: children[0].clone())
///         Empty()
///       }
///       Empty()
///     }
///   }
/// }
/// ```
///
/// [`component::Any`]: struct.Any.html
/// [`element::Any`]: ../element/struct.Any.html
/// [`render!`]: ../macro.render.html
#[derive(Default)]
pub struct Embed {
  pub content: Content,
}

impl Component for Embed {
  fn render(&self) -> AnyElement {
    match &self.content {
      Content::Component(component) => component.render(),
      Content::Element(element) => Clone::clone(element),
    }
  }
}
