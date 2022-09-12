//! Structures for working with colors and text modifiers.

use tui::style::Style as TuiStyle;
pub use tui::style::{Color, Modifier};

/// Styles that can apply to anything drawn on the screen.
///
/// Styles are composed of foreground colors, background colors,
/// and text modifiers. These fields are optional (modifiers have `Modifier::NONE`),
/// and can be merged together when multiple styles are applied.
///
/// `Style` also conveniently implements `From<Color>`, which creates a style
/// with the provided color as the froeground color.
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

/// `Convert` a color into a `Style` with a specific foreground color.
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
