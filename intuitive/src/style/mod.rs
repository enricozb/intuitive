use crossterm::style::ContentStyle as CrosstermStyle;
pub use crossterm::style::{Attribute, Attributes, Color};

/// The possible styles of a cell.
///
/// This can be constructed through [`From<Color>`], [`From<Attribute>`],
/// or [`From<[Attribute; N]>`](#impl-From<%5BAttribute%3B%20N%5D>-for-Style).
///
/// [`From<Color>`]: #impl-From<Color>-for-Style
/// [`From<Attribute>`]: #impl-From<Attribute>-for-Style
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style(pub CrosstermStyle);

/// Defaults to [`Color::Reset`] for the colors.
impl Default for Style {
  fn default() -> Self {
    Self(CrosstermStyle {
      foreground_color: Some(Color::Reset),
      background_color: Some(Color::Reset),
      underline_color: Some(Color::Reset),
      attributes: Attributes::default(),
    })
  }
}

impl From<Style> for CrosstermStyle {
  fn from(style: Style) -> Self {
    style.0
  }
}

/// Converts to a [`Style`] with `color` as its foreground color.
impl From<Color> for Style {
  fn from(color: Color) -> Self {
    Self(CrosstermStyle {
      foreground_color: Some(color),
      ..Default::default()
    })
  }
}

/// Converts to a [`Style`] with `attribute` as its attributes.
impl From<Attribute> for Style {
  fn from(attribute: Attribute) -> Self {
    Self(CrosstermStyle {
      attributes: attribute.into(),
      ..Default::default()
    })
  }
}

/// Converts to a [`Style`] with `attributes` as its attributes.
impl<const N: usize> From<[Attribute; N]> for Style {
  fn from(attributes: [Attribute; N]) -> Self {
    Self(CrosstermStyle {
      attributes: attributes.as_slice().into(),
      ..Default::default()
    })
  }
}
