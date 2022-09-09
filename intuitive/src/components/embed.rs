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
