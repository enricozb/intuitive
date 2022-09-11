//! Structures for working with colors and text modifiers.

use tui::style::Style as TuiStyle;
pub use tui::style::{Color, Modifier};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Style(TuiStyle);

impl Style {
  pub fn new(fg: Option<Color>, bg: Option<Color>, modifier: Modifier) -> Self {
    Self(TuiStyle {
      fg,
      bg,
      add_modifier: modifier,
      sub_modifier: Modifier::empty(),
    })
  }
}

impl From<Color> for Style {
  fn from(color: Color) -> Self {
    Self::new(Some(color), None, Modifier::empty())
  }
}

impl From<Style> for TuiStyle {
  fn from(style: Style) -> Self {
    style.0
  }
}
