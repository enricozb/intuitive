//! Cell-based back-buffer plus a `Region` for scoped writes.

use crate::geometry::{Point, Rectangle, Size};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Color {
  #[default]
  Reset,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,
  DarkGrey,
}

/// The unit of content in a `Buffer`.
///
/// ```
/// # use intuitive::buffer::{Cell, Color, Style};
/// let style = Style::new().fg(Color::Red).bg(Color::Yellow).bold(true);
/// Cell::styled('x', style);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
  pub ch: char,
  pub style: Style,
}

impl Cell {
  /// Equivalent to `Cell::default()` but available in `const` context.
  pub const BLANK: Self = Self {
    ch: ' ',
    style: Style::RESET,
  };

  #[must_use]
  pub fn new(ch: char) -> Self {
    Self { ch, ..Self::default() }
  }

  #[must_use]
  pub fn styled(ch: char, style: Style) -> Self {
    Self { ch, style }
  }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Style {
  pub fg: Color,
  pub bg: Color,
  pub bold: bool,
}

impl Style {
  /// Equivalent to `Style::new()` but available in `const` context.
  pub const RESET: Self = Self {
    fg: Color::Reset,
    bg: Color::Reset,
    bold: false,
  };

  /// Equivalent to `Style::RESET`.
  #[must_use]
  pub const fn new() -> Self {
    Self::RESET
  }

  /// Replace this cell's foreground color.
  #[must_use]
  pub const fn fg(mut self, fg: Color) -> Self {
    self.fg = fg;
    self
  }

  /// Replace this cell's background color.
  #[must_use]
  pub const fn bg(mut self, bg: Color) -> Self {
    self.bg = bg;
    self
  }

  /// Replace this cell's bold flag.
  #[must_use]
  pub const fn bold(mut self, bold: bool) -> Self {
    self.bold = bold;
    self
  }
}

/// Convenience enum which implements `into<`[`Signal`]`>`.
pub enum Decoration {
  Bold,
}

impl From<Decoration> for Style {
  fn from(decoration: Decoration) -> Self {
    match decoration {
      Decoration::Bold => Style::new().bold(true),
    }
  }
}

impl From<Color> for Style {
  fn from(color: Color) -> Self {
    Self::new().fg(color)
  }
}

/// Row-major grid of cells.
#[derive(Clone, Debug)]
pub struct Buffer {
  size: Size,
  cells: Vec<Cell>,
}

impl Buffer {
  #[must_use]
  pub fn new(size: Size) -> Self {
    Self {
      size,
      cells: vec![Cell::BLANK; size.area()],
    }
  }

  #[must_use]
  pub fn size(&self) -> Size {
    self.size
  }

  #[must_use]
  pub fn rect(&self) -> Rectangle {
    self.size.into()
  }

  /// Reinitializes the buffer with a new `size`.
  ///
  /// If `self.size == size`, this is a noop.
  pub fn resize(&mut self, size: Size) {
    if size == self.size {
      return;
    }

    *self = Self::new(size);
  }

  fn idx(&self, p: impl Into<Point>) -> Option<usize> {
    let Point { x, y } = p.into();
    if x < self.size.w && y < self.size.h {
      Some(usize::from(y) * usize::from(self.size.w) + usize::from(x))
    } else {
      None
    }
  }

  #[must_use]
  pub fn get(&self, p: impl Into<Point>) -> Option<&Cell> {
    self.idx(p).map(|i| &self.cells[i])
  }

  pub fn set(&mut self, p: impl Into<Point>, cell: Cell) {
    if let Some(i) = self.idx(p) {
      self.cells[i] = cell;
    }
  }

  /// A mutable view specified by `rect`. Areas beyond the buffer are clipped.
  pub fn region(&mut self, rect: Rectangle) -> Region<'_> {
    Region {
      rect: rect.clip(self.size),
      buffer: self,
    }
  }

  /// Compute the set of cells where `self` differs from `prev`. Returned
  /// as a vec of (position, current cell), in scan order.
  ///
  /// If buffer sizes differ, this returns every cell of `self` (caller
  /// should treat that as a full repaint).
  #[must_use]
  pub fn diff<'a>(&'a self, prev: &Buffer) -> Vec<(Point, &'a Cell)> {
    // TODO(enricozb): check if this is efficient, specifically this
    // statement that the runtime treats anything as a "full repaint".
    if self.size != prev.size {
      let mut out = Vec::with_capacity(self.cells.len());
      for y in 0..self.size.h {
        for x in 0..self.size.w {
          let i = usize::from(y) * usize::from(self.size.w) + usize::from(x);
          out.push((Point::new(x, y), &self.cells[i]));
        }
      }
      return out;
    }
    let mut out = Vec::new();
    for y in 0..self.size.h {
      for x in 0..self.size.w {
        let i = usize::from(y) * usize::from(self.size.w) + usize::from(x);
        if self.cells[i] != prev.cells[i] {
          out.push((Point::new(x, y), &self.cells[i]));
        }
      }
    }
    out
  }
}

/// Scoped mutable view into a sub-rectangle of a `Buffer`.
pub struct Region<'a> {
  rect: Rectangle,
  buffer: &'a mut Buffer,
}

impl Region<'_> {
  #[must_use]
  pub fn rect(&self) -> Rectangle {
    self.rect
  }

  #[must_use]
  pub fn size(&self) -> Size {
    self.rect.size
  }

  /// Write `cell` to `self.rect`-relative point `p`.
  ///
  /// Out-of-bounds writes are ignored.
  pub fn set(&mut self, p: impl Into<Point>, cell: Cell) {
    let Point { x, y } = p.into();
    if x >= self.rect.size.w || y >= self.rect.size.h {
      return;
    }
    let abs = Point::new(self.rect.pos.x + x, self.rect.pos.y + y);
    self.buffer.set(abs, cell);
  }

  /// Write `s` starting at `self.rect`-relative point `p`, with the `fg` color
  /// and `bold`. Truncated at the right edge of the region.
  pub fn write_str(&mut self, p: impl Into<Point>, s: &str, style: Style) {
    let Point { x, y } = p.into();
    if y >= self.rect.size.h {
      return;
    }
    let mut cur_x = x;
    for ch in s.chars() {
      if cur_x >= self.rect.size.w {
        break;
      }
      self.set((cur_x, y), Cell::styled(ch, style));
      cur_x = cur_x.saturating_add(1);
    }
  }

  /// Fill the entire region with `cell`.
  pub fn fill(&mut self, cell: Cell) {
    for y in 0..self.rect.size.h {
      for x in 0..self.rect.size.w {
        self.set((x, y), cell);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn buffer_set_get() {
    let mut b = Buffer::new(Size::new(4, 3));
    b.set(Point::new(1, 2), Cell::new('x'));
    assert_eq!(b.get((1, 2)), Some(&Cell::new('x')));
    assert_eq!(b.get((0, 0)), Some(&Cell::BLANK));
    assert_eq!(b.get((4, 0)), None);
  }

  #[test]
  fn region_writes_clipped() {
    let mut b = Buffer::new(Size::new(5, 2));
    let mut r = b.region(Rectangle::new(Point::new(1, 0), Size::new(3, 2)));
    r.write_str((0, 0), "hello", Style::new().fg(Color::Red));
    // visible portion is 'hel' starting at x=1.
    assert_eq!(b.get((0, 0)).unwrap().ch, ' ');
    assert_eq!(b.get((1, 0)).unwrap().ch, 'h');
    assert_eq!(b.get((2, 0)).unwrap().ch, 'e');
    assert_eq!(b.get((3, 0)).unwrap().ch, 'l');
    assert_eq!(b.get((4, 0)).unwrap().ch, ' ');
  }

  #[test]
  fn region_clipped_to_buffer() {
    let mut b = Buffer::new(Size::new(4, 4));
    // requested rect extends past buffer; should be clipped.
    let mut r = b.region(Rectangle::new(Point::new(2, 2), Size::new(10, 10)));
    assert_eq!(r.size(), Size::new(2, 2));
    r.fill(Cell::new('#'));
    for y in 0..4 {
      for x in 0..4 {
        let want = if x >= 2 && y >= 2 { '#' } else { ' ' };
        assert_eq!(b.get((x, y)).unwrap().ch, want, "at ({x},{y})");
      }
    }
  }

  #[test]
  fn diff_basic() {
    let prev = Buffer::new(Size::new(3, 2));
    let mut curr = Buffer::new(Size::new(3, 2));
    curr.set((1, 0), Cell::new('x'));
    curr.set((2, 1), Cell::new('y'));
    let d = curr.diff(&prev);
    assert_eq!(d.len(), 2);
    assert_eq!(d[0].0, Point::new(1, 0));
    assert_eq!(d[0].1.ch, 'x');
    assert_eq!(d[1].0, Point::new(2, 1));
    assert_eq!(d[1].1.ch, 'y');
  }

  #[test]
  fn diff_size_change_is_full() {
    let prev = Buffer::new(Size::new(2, 2));
    let curr = Buffer::new(Size::new(3, 2));
    let d = curr.diff(&prev);
    assert_eq!(d.len(), 6);
  }

  #[test]
  fn resize_discards_contents() {
    let mut b = Buffer::new(Size::new(3, 3));
    b.set((0, 0), Cell::new('a'));
    b.set((2, 2), Cell::new('z'));
    b.resize(Size::new(2, 2));
    assert_eq!(b.size(), Size::new(2, 2));
    assert_eq!(b.get((0, 0)).unwrap().ch, ' ');
    assert_eq!(b.get((1, 1)).unwrap().ch, ' ');
    assert!(b.get((2, 2)).is_none());
  }
}
