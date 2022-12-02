use crate::{
  buffer::region::Region,
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
  utils::{
    geometry::{Position, Size},
    layout::Amount,
  },
};

#[derive(Clone, Default)]
/// Renders its child at a fixed width and/or height.
///
/// The parameters `width` and `height` default to [`Amount::default`].
pub struct Fixed {
  pub width: Amount,
  pub height: Amount,

  pub children: [AnyElement; 1],
}

impl Component for Fixed {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Fixed {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
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

    self.children[0].draw(&mut region)?;

    Ok(())
  }
}
