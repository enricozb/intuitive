#[derive(Clone, Copy)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

impl From<(u16, u16)> for Size {
  fn from((width, height): (u16, u16)) -> Self {
    Self { width, height }
  }
}

pub struct Position {
  pub x: u16,
  pub y: u16,
}

impl Position {
  /// Returns a [`Position`] given an index and a [`Size`].
  ///
  /// This maps a 1-dimensional value to a 2-dimensional value by "wrapping" `idx` within the `size`
  /// going left-to-right and top-to-bottom.
  #[must_use]
  pub fn from_idx(idx: u16, size: Size) -> Self {
    Self {
      x: idx % size.width,
      y: idx / size.width,
    }
  }
}
