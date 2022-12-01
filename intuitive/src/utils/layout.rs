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
  Fixed(u16),
  /// A percentage as an integer, ranging from 0-100.
  Percentage(u8),
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
