use crate::{
  buffer::{draw::Draw, region::Region},
  components::Component,
  element::{Any as AnyElement, Children, Element},
  error::Result,
  style::Style,
  utils::{
    geometry::{Axis, Position, Size},
    layout::Alignment,
  },
};

/// Wraps its child in a border and a title.
#[derive(Clone, Default)]
pub struct Section {
  pub title: String,

  /// [`Style`] of the border.
  pub border: Style,

  /// [`Alignment`] of the title.
  pub alignment: Alignment,

  pub children: Children<1>,
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

    let width_sub_2 = size.width.saturating_sub(2);
    let height_sub_2 = size.height.saturating_sub(2);

    region.set_char(Position { x: min_x, y: min_y }, '╭', self.border);
    region.set_char(Position { x: max_x, y: min_y }, '╮', self.border);
    region.set_char(Position { x: max_x, y: max_y }, '╯', self.border);
    region.set_char(Position { x: min_x, y: max_y }, '╰', self.border);

    region.repeat_char(Axis::Horizontal, Position { x: min_x + 1, y: min_y }, '─', self.border, width_sub_2);
    region.repeat_char(Axis::Horizontal, Position { x: min_x + 1, y: max_y }, '─', self.border, width_sub_2);
    region.repeat_char(Axis::Vertical, Position { x: min_x, y: min_y + 1 }, '│', self.border, height_sub_2);
    region.repeat_char(Axis::Vertical, Position { x: max_x, y: min_y + 1 }, '│', self.border, height_sub_2);

    #[allow(clippy::cast_possible_truncation)]
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

    region.write_string(Axis::Horizontal, title_position, &self.title, Style::default());

    let mut region = region.narrow(
      Position { x: 1, y: 1 },
      Size {
        width: width_sub_2,
        height: height_sub_2,
      },
    )?;

    self.children[0].draw(&mut region)?;

    Ok(())
  }
}
