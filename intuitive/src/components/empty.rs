use crate::{components::Component, element::Any as AnyElement, render::context::Context};

/// Renders an empty element.
#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self, _context: &mut Context) -> AnyElement {
    AnyElement::default()
  }
}
