use crate::{
  buffer::{draw::Draw, region::Region},
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
  utils::{
    geometry::{Axis, Position, Size},
    layout::Alignment,
  },
};

#[derive(Clone, Default)]
/// Wraps its child in a border and a title.
pub struct Section {
  pub title: String,

  /// [`Alignment`] of the title.
  pub alignment: Alignment,

  pub children: [AnyElement; 1],
}

impl Component for Section {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl Element for Section {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    let size = region.size();
    let (min_x, min_y) = (0, 0);
    let (max_x, max_y) = (size.width.saturating_sub(1), size.height.saturating_sub(1));

    let width_minus_2 = size.width.saturating_sub(2);
    let height_minus_2 = size.height.saturating_sub(2);

    region.set_char(Position { x: min_x, y: min_y }, '╭');
    region.set_char(Position { x: max_x, y: min_y }, '╮');
    region.set_char(Position { x: max_x, y: max_y }, '╯');
    region.set_char(Position { x: min_x, y: max_y }, '╰');

    region.repeat_char(Axis::Horizontal, Position { x: min_x + 1, y: min_y }, '─', width_minus_2);
    region.repeat_char(Axis::Horizontal, Position { x: min_x + 1, y: max_y }, '─', width_minus_2);
    region.repeat_char(Axis::Vertical, Position { x: min_x, y: min_y + 1 }, '│', height_minus_2);
    region.repeat_char(Axis::Vertical, Position { x: max_x, y: min_y + 1 }, '│', height_minus_2);

    let title_position = match self.alignment {
      Alignment::Left => Position { x: 1, y: 0 },
      Alignment::Center => Position {
        x: region.size().width.saturating_sub(self.title.len() as u16) / 2,
        y: 0,
      },
      Alignment::Right => Position {
        x: region.size().width.saturating_sub(self.title.len() as u16 + 1),
        y: 0,
      },
    };

    region.write_string(Axis::Horizontal, title_position, &self.title);

    let mut region = region.narrow(
      Position { x: 1, y: 1 },
      Size {
        width: width_minus_2,
        height: height_minus_2,
      },
    )?;

    self.children[0].draw(&mut region)?;

    Ok(())
  }
}
