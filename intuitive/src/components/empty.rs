use crate::{components::Component, element::Any as AnyElement};

/// Renders an empty element.
#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self) -> AnyElement {
    AnyElement::default()
  }
}
