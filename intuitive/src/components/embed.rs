use crate::{
  components::Component,
  draw::{Context as DrawContext, Region},
  element::{Any as AnyElement, Children, Element},
  error::Result,
  render::Context as RenderContext,
};

/// Renders its child.
#[derive(Clone, Default)]
pub struct Embed {
  pub children: Children<1>,
}

impl Component for Embed {
  fn render(&self, context: &mut RenderContext) -> AnyElement {
    AnyElement::new(context.current_component_id(), self.clone())
  }
}

impl Element for Embed {
  fn draw<'a>(&self, context: &mut DrawContext, region: &'a mut Region<'a>) -> Result<()> {
    context.draw(&self.children[0], region)
  }
}
