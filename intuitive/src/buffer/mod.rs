pub mod draw;
pub mod region;

use std::{
  io::{Stdout, Write},
  iter,
};

use crossterm::{cursor::MoveTo, queue, style::Print};

use self::draw::Draw;
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
  pub(crate) fn draw_diff(&self, other: &Self, stdout: &mut Stdout) -> Result<()> {
    for (pos, cell) in self.diffs(other).cells {
      queue!(stdout, MoveTo(pos.x, pos.y))?;

      match cell {
        None | Some(Cell { chr: None, .. }) => queue!(stdout, Print(" "))?,
        Some(Cell { chr: Some(c), .. }) => queue!(stdout, Print(c))?,
      };
    }

    stdout.flush()?;

    Ok(())
  }
}

impl Draw for Buffer {
  fn set_option_cell(&mut self, position: Position, cell: Option<Cell>) -> bool {
    let idx = usize::from(position.into_idx(self.size));
    let in_bounds = idx < self.data.len();

    if in_bounds {
      self.data[idx] = cell;
    }

    in_bounds
  }

  fn size(&self) -> Size {
    self.size
  }
}

/// A single cell within a [`Buffer`].
#[derive(PartialEq, Eq, Debug)]
pub struct Cell {
  chr: Option<char>,
}

/// Differences between two [`Buffer`]s.
struct Diffs<'a> {
  cells: Vec<(Position, &'a Option<Cell>)>,
}
