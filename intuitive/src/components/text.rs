use crate::{
  buffer::{draw::Draw, region::Region},
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
  utils::geometry::{Axis, Position},
};

#[derive(Clone, Default)]
/// Displays simple text.
pub struct Text {
  pub text: String,
}

impl Component for Text {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Text {
  fn draw(&self, region: &mut Region) -> Result<()> {
    region.write_string(Axis::Horizontal, Position::default(), &self.text);

    Ok(())
  }
}
