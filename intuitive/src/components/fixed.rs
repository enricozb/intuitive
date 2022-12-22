use crate::{
  components::Component,
  draw::{Context as DrawContext, Region},
  element::{Any as AnyElement, Children, Element},
  error::Result,
  render::Context as RenderContext,
  utils::{
    geometry::{Position, Size},
    layout::Amount,
  },
};

/// Renders its child at a fixed width and/or height.
///
/// The parameters `width` and `height` default to [`Amount::default`].
#[derive(Clone, Default)]
pub struct Fixed {
  pub width: Amount,
  pub height: Amount,

  pub children: Children<1>,
}

impl Component for Fixed {
  fn render(&self, context: &mut RenderContext) -> AnyElement {
    AnyElement::new(context.current_component_id(), self.clone())
  }
}

impl Element for Fixed {
  fn draw<'a>(&self, context: &mut DrawContext, region: &'a mut Region<'a>) -> Result<()> {
    let size = region.size();

    let width = self.width.of(size.width);
    let height = self.height.of(size.height);

    let mut region = region.narrow(
      Position {
        x: size.width.saturating_sub(width) / 2,
        y: size.height.saturating_sub(height) / 2,
      },
      Size { width, height },
    )?;

    context.draw(&self.children[0], &mut region)?;

    Ok(())
  }
}
