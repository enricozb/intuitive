use crate::{
  buffer::region::Region,
  components::Component,
  element::{Any as AnyElement, Children, Element},
  error::Result,
  utils::{
    geometry::{Position, Size},
    layout::Amount,
  },
};

#[derive(Clone)]
/// Renders its child with padding around it.
///
/// If the provided amount is a [`Amount::Percentage`], then the horizontal padding will be a
/// percentage of the width of the terminal, and the vertical padding will be a percentage
/// of its hight.
pub struct Padding {
  /// The amount of padding.
  pub amount: Amount,

  pub children: Children<1>,
}

impl Default for Padding {
  fn default() -> Self {
    Self {
      amount: Amount::Fixed(2),
      children: Default::default(),
    }
  }
}

impl Component for Padding {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Padding {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    let size = region.size();
    let padding_x = self.amount.of(size.width);
    let padding_y = self.amount.of(size.height);

    let mut region = region.narrow(
      Position {
        x: padding_x,
        y: padding_y,
      },
      Size {
        width: size.width.saturating_sub(padding_x * 2),
        height: size.height.saturating_sub(padding_y * 2),
      },
    )?;

    self.children[0].draw(&mut region)?;

    Ok(())
  }
}
