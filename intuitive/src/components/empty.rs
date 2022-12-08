use crate::{components::Component, element::Any as AnyElement, render::manager::Manager as RenderManager};

/// Renders an empty element.
#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self, _render: &mut RenderManager) -> AnyElement {
    AnyElement::default()
  }
}
