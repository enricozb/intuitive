use super::Cell;
#[allow(unused)]
use super::{region::Region, Buffer};
use crate::utils::geometry::{Axis, Position, Size};

/// Types that can be drawn onto, mostly for [`Buffer`] and [`Region`].
pub trait Draw {
  /// Sets an `Option<`[`Cell`]`>` at a given [`Position`].
  fn set_option_cell(&mut self, position: Position, cell: Option<Cell>) -> bool;

  /// Sets a [`Cell`] at a given [`Position`].
  fn set_cell(&mut self, position: Position, cell: Cell) -> bool {
    self.set_option_cell(position, Some(cell))
  }

  /// Unsets a [`Cell`] at a given [`Position`].
  fn unset_cell(&mut self, position: Position) -> bool {
    self.set_option_cell(position, None)
  }

  /// Sets a [`char`] at a given [`Position`].
  fn set_char(&mut self, position: Position, chr: char) -> bool {
    self.set_cell(position, Cell { chr: Some(chr) })
  }

  /// Writes a string at a given [`Position`] and [`Axis`].
  ///
  /// For [`Axis::Horizontal`], the provided position is the left-most character of the string,
  /// and for [`Axis::Vertical`], the provided position is the top-most character of the string.
  fn write_string(&mut self, axis: Axis, mut position: Position, string: &str) {
    for chr in string.chars() {
      if !self.set_cell(position, Cell { chr: Some(chr) }) {
        return;
      }

      match axis {
        Axis::Horizontal => position.x += 1,
        Axis::Vertical => position.y += 1,
      }
    }
  }

  /// Repeats a `char` at a given [`Position`] and [`Axis`] `n` times.
  ///
  /// For [`Axis::Horizontal`], the provided position is the left-most character of the string,
  /// and for [`Axis::Vertical`], the provided position is the top-most character of the string.
  fn repeat_char(&mut self, axis: Axis, mut position: Position, chr: char, n: u16) {
    for _ in 0..n {
      if !self.set_cell(position, Cell { chr: Some(chr) }) {
        return;
      }

      match axis {
        Axis::Horizontal => position.x += 1,
        Axis::Vertical => position.y += 1,
      }
    }
  }

  /// Returns the size of the region being drawn onto.
  #[must_use]
  fn size(&self) -> Size;
}
