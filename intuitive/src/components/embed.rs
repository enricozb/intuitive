use crate::{
  buffer::region::Region,
  components::Component,
  element::{Any as AnyElement, Children, Element},
  error::Result,
  render::Manager as RenderManager,
};

/// Renders its child.
#[derive(Clone, Default)]
pub struct Embed {
  pub children: Children<1>,
}

impl Component for Embed {
  fn render(&self, _render: &mut RenderManager) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Embed {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    self.children[0].draw(region)
  }
}
