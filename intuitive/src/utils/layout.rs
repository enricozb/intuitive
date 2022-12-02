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

#[derive(Clone, Copy)]
pub enum Amount {
  /// A fixed amount in cells.
  Fixed(u16),
  /// A percentage as an integer, ranging from 0-100.
  Percentage(u8),
}

impl Default for Amount {
  /// Defaults to 100%.
  fn default() -> Self {
    Self::Percentage(100)
  }
}

impl Amount {
  /// Returns the fixed amount if `self` is [`Amount::Fixed`], or the percentage
  /// of `amount` if `self` is [`Amount::Percentage`].
  #[must_use]
  pub fn of(&self, amount: u16) -> u16 {
    match self {
      Self::Fixed(fixed) => *fixed,
      Self::Percentage(pct) => u16::from(*pct) * amount / 100,
    }
  }
}

/// Controls how much space a child of a flex component should occupy.
#[derive(Clone, Copy)]
pub enum Flex {
  /// A fixed amount of space in cells.
  Fixed(u16),
  /// A relative amount of height or width.
  Grow(u16),
}

impl Default for Flex {
  /// Defaults to [`Flex::Grow`]`(1)`.
  fn default() -> Self {
    Self::Grow(1)
  }
}

impl Flex {
  /// Returns the inner value of [`Self::Fixed`], or `0`.
  #[must_use]
  pub fn fixed(&self) -> u16 {
    match self {
      Self::Fixed(fixed) => *fixed,
      Self::Grow(_) => 0,
    }
  }

  /// Returns the inner value of [`Self::Grow`], or `0`.
  #[must_use]
  pub fn grow(&self) -> u16 {
    match self {
      Self::Fixed(_) => 0,
      Self::Grow(rel) => *rel,
    }
  }
}

/// Controls the direction a flex component should layout its children.
#[derive(Clone, Copy)]
pub enum FlexDirection {
  Row,
  Column,
}
