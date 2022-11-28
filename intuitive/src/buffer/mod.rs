use std::{io::Stdout, iter};

use crossterm::{cursor::MoveTo, queue, style::Print};

use crate::{
  error::Result,
  utils::geometry::{Position, Size},
};

/// A buffer of text that can be drawn onto a terminal.
pub struct Buffer {
  size: Size,
  data: Vec<Option<Cell>>,
}

impl Buffer {
  /// Creates a new [`Buffer`].
  pub fn new<S: Into<Size>>(size: S) -> Self {
    let size = size.into();

    let mut data = Vec::new();
    data.resize_with((size.width * size.height) as usize, Option::default);

    Self { size, data }
  }

  #[must_use]
  /// Returns the [`Size`] of the [`Buffer`].
  fn size(&self) -> Size {
    self.size
  }

  /// Computes the differences between `other` and `self`. Specifically, return [`Cell`]s from `self`
  /// when they differ from `other`.
  fn diffs(&self, other: &Self) -> Diffs {
    let mut diffs = Vec::new();

    for (index, (new, old)) in iter::zip(&self.data, &other.data).enumerate() {
      if new != old {
        #[allow(clippy::cast_possible_truncation)]
        diffs.push((Position::from_idx(index as u16, self.size), new));
      }
    }

    Diffs { cells: diffs }
  }

  /// Draws the difference between `other` and `self` onto `stdout`.
  fn draw_diff(&self, other: &Self, stdout: &mut Stdout) -> Result<()> {
    for (pos, cell) in self.diffs(other).cells {
      queue!(stdout, MoveTo(pos.x, pos.y))?;

      match cell {
        None | Some(Cell { content: None, .. }) => queue!(stdout, Print(" "))?,
        Some(Cell { content: Some(c), .. }) => queue!(stdout, Print(c))?,
      };
    }

    Ok(())
  }
}

/// A single cell within a [`Buffer`].
#[derive(PartialEq, Eq)]
pub struct Cell {
  content: Option<char>,
}

/// Differences between two [`Buffer`]s.
struct Diffs<'a> {
  cells: Vec<(Position, &'a Option<Cell>)>,
}
