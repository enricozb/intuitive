use tui::layout::Alignment as TuiAlignment;

/// Control how the text in a [`Text`] component is aligned.
///
/// [`Text`]: ../struct.Text.html
#[derive(Clone, Copy)]
pub enum Alignment {
  Left,
  Center,
  Right,
}

impl Default for Alignment {
  fn default() -> Self {
    Self::Left
  }
}

impl From<Alignment> for TuiAlignment {
  fn from(alignment: Alignment) -> Self {
    match alignment {
      Alignment::Left => Self::Left,
      Alignment::Center => Self::Center,
      Alignment::Right => Self::Right,
    }
  }
}
