use super::Element;

/// The empty [`Element`]. It draws nothing.
pub struct Empty;

impl Element for Empty {
  fn draw(&self) {}
}
