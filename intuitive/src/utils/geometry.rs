use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

impl From<(u16, u16)> for Size {
  fn from((width, height): (u16, u16)) -> Self {
    Self { width, height }
  }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Position {
  pub x: u16,
  pub y: u16,
}

impl Add for Position {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self {
      x: self.x + rhs.x,
      y: self.y + rhs.y,
    }
  }
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

  /// Returns a `u16` given a [`Size`].
  ///
  /// This maps the 2-dimensional value of a [`Position`] to a 1-dimensional value by "unwrapping" `self` within the `size`
  /// going left-to-right and top-to-bottom.
  #[must_use]
  pub fn into_idx(&self, size: Size) -> u16 {
    self.x + size.width * self.y
  }
}

#[derive(Clone, Copy)]
pub enum Axis {
  Horizontal,
  Vertical,
}
