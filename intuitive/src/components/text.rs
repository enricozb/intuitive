use crate::{
  buffer::{draw::Draw, region::Region},
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
  utils::{
    geometry::{Axis, Position},
    layout::Alignment,
  },
};

/// Displays simple text.
#[derive(Clone, Default)]
pub struct Text {
  pub text: String,
  pub alignment: Alignment,
}

impl Component for Text {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Text {
  fn draw(&self, region: &mut Region) -> Result<()> {
    let position = match self.alignment {
      Alignment::Left => Position::default(),
      Alignment::Center => Position {
        x: region.size().width.saturating_sub(self.text.len() as u16) / 2,
        y: 0,
      },
      Alignment::Right => Position {
        x: region.size().width.saturating_sub(self.text.len() as u16),
        y: 0,
      },
    };

    region.write_string(Axis::Horizontal, position, &self.text);

    Ok(())
  }
}
