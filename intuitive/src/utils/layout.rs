#[derive(Clone, Copy)]
pub enum Alignment {
  Left,
  Center,
  Right,
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
  pub fn of(&self, amount: u16) -> u16 {
    match self {
      Self::Fixed(fixed) => *fixed,
      Self::Percentage(pct) => (*pct as u16) * amount / 100,
    }
  }
}
