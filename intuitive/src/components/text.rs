use crate::{
  buffer::region::Region,
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
};

#[derive(Clone, Default)]
pub struct Text {
  pub text: String,
}

impl Component for Text {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Text {
  fn draw(&self, region: Region) -> Result<()> {
    Ok(())
  }
}
