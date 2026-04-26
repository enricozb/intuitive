//! Geometry primitives: positions, sizes, rectangles.

use derive_more::{Constructor, From};

#[derive(Constructor, From, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Point {
  pub x: u16,
  pub y: u16,
}

#[derive(Constructor, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Size {
  pub w: u16,
  pub h: u16,
}

impl Size {
  #[must_use]
  pub fn area(self) -> usize {
    (self.w as usize) * (self.h as usize)
  }
}

#[derive(Constructor, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Rectangle {
  pub pos: Point,
  pub size: Size,
}

impl Rectangle {
  #[must_use]
  pub fn left(self) -> u16 {
    self.pos.x
  }

  #[must_use]
  pub fn top(self) -> u16 {
    self.pos.y
  }

  #[must_use]
  pub fn right(self) -> u16 {
    self.pos.x.saturating_add(self.size.w)
  }

  #[must_use]
  pub fn bottom(self) -> u16 {
    self.pos.y.saturating_add(self.size.h)
  }

  #[must_use]
  pub fn contains(self, p: Point) -> bool {
    p.x >= self.left() && p.x < self.right() && p.y >= self.top() && p.y < self.bottom()
  }

  /// Clip this rectangle to fit in `size`, implicitly at position `(0, 0)`.
  #[must_use]
  pub fn clip(self, size: Size) -> Self {
    let x = self.pos.x.min(size.w);
    let y = self.pos.y.min(size.h);
    let w = self.size.w.min(size.w.saturating_sub(x));
    let h = self.size.h.min(size.h.saturating_sub(y));

    Self::new(Point::new(x, y), Size::new(w, h))
  }

  /// True if `self` and `other` share at least one cell. Zero-area rectangles
  /// never intersect anything (including each other).
  #[must_use]
  pub fn intersects(self, other: Rectangle) -> bool {
    if self.size.w == 0 || self.size.h == 0 || other.size.w == 0 || other.size.h == 0 {
      return false;
    }

    self.left() < other.right()
      && other.left() < self.right()
      && self.top() < other.bottom()
      && other.top() < self.bottom()
  }

  /// Split this rectangle horizontally (along the x-axis) using per-slot
  /// [`Flex`] rules. `Flex::Cell` slots are allocated first (clipped to the
  /// available width in declaration order), and the remaining width is divided
  /// among `Flex::Frac` slots proportionally to their weights. Slices fully
  /// tile this rectangle when at least one `Frac` with non-zero weight is
  /// present and total `Cell`s do not exceed the width; otherwise any leftover
  /// cells are simply not assigned (no slice extends past `self.right()`).
  ///
  /// Returns one rectangle per `flex` entry, in declaration order. Never
  /// panics, even if the requested layout cannot fit.
  #[must_use]
  pub fn split_h_flex(self, flex: &[Flex]) -> Vec<Rectangle> {
    let widths = distribute(u32::from(self.size.w), flex);
    let mut out = Vec::with_capacity(flex.len());
    let mut x = u32::from(self.pos.x);
    let max_right = u32::from(self.pos.x).saturating_add(u32::from(self.size.w));
    for w in widths {
      let clamped_x = x.min(max_right);
      let x16 = u16::try_from(clamped_x).unwrap_or(u16::MAX);
      let w16 = u16::try_from(w).unwrap_or(u16::MAX);
      out.push(Rectangle::new(Point::new(x16, self.pos.y), Size::new(w16, self.size.h)));
      x = x.saturating_add(w);
    }
    out
  }

  /// Split this rectangle vertically (along the y-axis) using per-slot [`Flex`]
  /// rules. See [`split_h_flex`] for semantics.
  ///
  /// [`split_h_flex`]: Self::split_h_flex
  #[must_use]
  pub fn split_v_flex(self, flex: &[Flex]) -> Vec<Rectangle> {
    let heights = distribute(u32::from(self.size.h), flex);
    let mut out = Vec::with_capacity(flex.len());
    let mut y = u32::from(self.pos.y);
    let max_bottom = u32::from(self.pos.y).saturating_add(u32::from(self.size.h));
    for h in heights {
      let clamped_y = y.min(max_bottom);
      let y16 = u16::try_from(clamped_y).unwrap_or(u16::MAX);
      let h16 = u16::try_from(h).unwrap_or(u16::MAX);
      out.push(Rectangle::new(Point::new(self.pos.x, y16), Size::new(self.size.w, h16)));
      y = y.saturating_add(h);
    }
    out
  }

  /// Shrink this rectangle inward by 1 cell on each edge. Useful for computing
  /// the inner rect of a 1-cell border. If a dimension is < 2, returns an empty
  /// rect at the same position.
  #[must_use]
  pub fn inset(self) -> Rectangle {
    if self.size.w < 2 || self.size.h < 2 {
      return Rectangle::new(self.pos, Size::new(0, 0));
    }

    Rectangle::new(
      Point::new(self.pos.x + 1, self.pos.y + 1),
      Size::new(self.size.w - 2, self.size.h - 2),
    )
  }
}

impl From<Size> for Rectangle {
  fn from(size: Size) -> Self {
    Self::new(Point::default(), size)
  }
}

/// Layout for a component along an axis, similar to [`flex` in CSS].
///
/// [`flex` in CSS]: https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Flexible_box_layout
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Flex {
  /// Proportional share of non-fixed space.
  Frac(usize),
  /// A fixed number of cells.
  Cell(u16),
}

impl Flex {
  fn frac(&self) -> Option<usize> {
    match *self {
      Self::Frac(f) => Some(f),
      Self::Cell(_) => None,
    }
  }

  fn cell(&self) -> Option<u16> {
    match *self {
      Self::Frac(_) => None,
      Self::Cell(c) => Some(c),
    }
  }
}

/// Nine-way (3×3) alignment of content, named after the cardinal directions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum Align {
  /// Top-left.
  #[default]
  NW,
  /// Top-center.
  N,
  /// Top-right.
  NE,
  /// Middle-left.
  W,
  /// Center of the container.
  Center,
  /// Middle-right.
  E,
  /// Bottom-left.
  SW,
  /// Bottom-center.
  S,
  /// Bottom-right.
  SE,
}

impl Align {
  /// Compute the top-left position of `content` inside `container` with the
  /// given alignment.
  ///
  /// If `content` is larger than `container` along an axis, that axis is
  /// pinned to 0 (i.e. the overflow extends past the bottom-right edge).
  #[must_use]
  pub fn place(self, content: Size, container: Size) -> Point {
    let dx = container.w.saturating_sub(content.w);
    let dy = container.h.saturating_sub(content.h);
    let (x, y) = match self {
      Align::NW => (0, 0),
      Align::N => (dx / 2, 0),
      Align::NE => (dx, 0),
      Align::W => (0, dy / 2),
      Align::Center => (dx / 2, dy / 2),
      Align::E => (dx, dy / 2),
      Align::SW => (0, dy),
      Align::S => (dx / 2, dy),
      Align::SE => (dx, dy),
    };

    Point::new(x, y)
  }
}

/// Compute per-slot allocations along one axis for a `Flex` layout.
///
/// `total` is the available extent (width or height) of the parent. Returns one
/// `u32` per `flex` entry, in declaration order. The sum of the returned values
/// is always `<= total`; it equals `total` exactly when there is at least one
/// `Frac` slot with non-zero weight *and* `Cell` requests do not exceed
/// `total`.
///
/// `Cell` slots are allocated first in declaration order; each gets
/// `min(request, remaining)`. After all `Cell`s are placed, the leftover is
/// split proportionally among `Frac` slots.
fn distribute(total: u32, flex: &[Flex]) -> Vec<u32> {
  let mut out = vec![0u32; flex.len()];

  // allocate `Cell`s, without exhausting `remaining`
  let mut remaining = total;
  for (i, f) in flex.iter().enumerate() {
    if let Some(want) = f.cell() {
      let give = remaining.min(u32::from(want));
      out[i] = give;
      remaining -= give;
    }
  }

  let fracs: Vec<(usize, usize)> = flex
    .iter()
    .enumerate()
    .filter_map(|(i, f)| Some((i, f.frac()?)))
    .collect();
  let total_weight: u64 = fracs.iter().map(|&(_, w)| w as u64).sum();

  if remaining == 0 || total_weight == 0 {
    return out;
  }

  // Allocate `Frac`s by computing the expected consumed cells for the "next"
  // `Frac`, and allocating the difference of the current and next. If we are
  // looking at the last `Frac` instance, the "next" consumed should be the
  // full remaining after considering the `Cell`s above.
  let mut consumed: u64 = 0;
  let mut weight_so_far: u64 = 0;
  let total_remaining = u64::from(remaining);
  for (idx_in_fracs, &(i, w)) in fracs.iter().enumerate() {
    weight_so_far += w as u64;
    let next_consumed = if idx_in_fracs == fracs.len() - 1 {
      total_remaining
    } else {
      (weight_so_far * total_remaining) / total_weight
    };
    out[i] = u32::try_from(next_consumed - consumed).unwrap_or(u32::MAX);
    consumed = next_consumed;
  }

  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn contains() {
    let r = Rectangle::new(Point::new(2, 3), Size::new(5, 4));
    assert!(r.contains(Point::new(2, 3)));
    assert!(r.contains(Point::new(6, 6)));
    assert!(!r.contains(Point::new(7, 6))); // right edge exclusive
    assert!(!r.contains(Point::new(6, 7))); // bottom edge exclusive
    assert!(!r.contains(Point::new(1, 3)));
  }

  #[test]
  fn inset() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(10, 5));
    assert_eq!(r.inset(), Rectangle::new(Point::new(1, 1), Size::new(8, 3)));
    let r = Rectangle::new(Point::new(0, 0), Size::new(1, 5));
    assert_eq!(r.inset().size, Size::new(0, 0));
  }

  #[test]
  fn align_nw_is_top_left() {
    let p = Align::NW.place(Size::new(3, 1), Size::new(10, 5));
    assert_eq!(p, Point::new(0, 0));
  }

  #[test]
  fn align_center() {
    let p = Align::Center.place(Size::new(4, 2), Size::new(10, 6));
    // dx=6, dy=4 -> (3, 2).
    assert_eq!(p, Point::new(3, 2));
  }

  #[test]
  fn align_se_is_bottom_right() {
    let p = Align::SE.place(Size::new(2, 1), Size::new(5, 3));
    // dx=3, dy=2.
    assert_eq!(p, Point::new(3, 2));
  }

  #[test]
  fn align_n_pins_to_top_centered_x() {
    let p = Align::N.place(Size::new(2, 1), Size::new(10, 4));
    // dx=8 -> 4; dy ignored, y=0.
    assert_eq!(p, Point::new(4, 0));
  }

  #[test]
  fn align_w_pins_to_left_centered_y() {
    let p = Align::W.place(Size::new(2, 1), Size::new(10, 5));
    // dy=4 -> 2; dx ignored, x=0.
    assert_eq!(p, Point::new(0, 2));
  }

  #[test]
  fn align_content_larger_pins_to_zero() {
    // Content wider than container: x saturates to 0.
    let p = Align::Center.place(Size::new(20, 10), Size::new(5, 3));
    assert_eq!(p, Point::new(0, 0));
  }

  #[test]
  fn split_h_flex_all_frac() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(10, 5));
    let parts = r.split_h_flex(&[Flex::Frac(1), Flex::Frac(1)]);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(5, 5)));
    assert_eq!(parts[1], Rectangle::new(Point::new(5, 0), Size::new(5, 5)));
  }

  #[test]
  fn split_h_flex_cell_reserves_first() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(10, 5));
    let parts = r.split_h_flex(&[Flex::Cell(3), Flex::Frac(1), Flex::Frac(1)]);
    assert_eq!(parts.len(), 3);
    // 3 reserved, leftover 7 split 1:1 -> 3 / 4 (last gets remainder).
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(3, 5)));
    assert_eq!(parts[1], Rectangle::new(Point::new(3, 0), Size::new(3, 5)));
    assert_eq!(parts[2], Rectangle::new(Point::new(6, 0), Size::new(4, 5)));
  }

  #[test]
  fn split_h_flex_cell_in_middle() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(10, 5));
    let parts = r.split_h_flex(&[Flex::Frac(1), Flex::Cell(4), Flex::Frac(1)]);
    // 4 reserved for middle cell, 6 remaining split 1:1 -> 3:3.
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(3, 5)));
    assert_eq!(parts[1], Rectangle::new(Point::new(3, 0), Size::new(4, 5)));
    assert_eq!(parts[2], Rectangle::new(Point::new(7, 0), Size::new(3, 5)));
  }

  #[test]
  fn split_h_flex_only_cells_fit() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(10, 5));
    let parts = r.split_h_flex(&[Flex::Cell(3), Flex::Cell(4)]);
    // No Fracs: leftover 3 cells go unallocated.
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(3, 5)));
    assert_eq!(parts[1], Rectangle::new(Point::new(3, 0), Size::new(4, 5)));
  }

  #[test]
  fn split_h_flex_cells_overflow_clipped() {
    // Total cell requests 8 in a 5-wide rect: first gets 5, second 0.
    let r = Rectangle::new(Point::new(0, 0), Size::new(5, 2));
    let parts = r.split_h_flex(&[Flex::Cell(3), Flex::Cell(5)]);
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(3, 2)));
    assert_eq!(parts[1], Rectangle::new(Point::new(3, 0), Size::new(2, 2)));
  }

  #[test]
  fn split_h_flex_cells_fill_exactly_no_frac_space() {
    // Cells take all available; Frac slot becomes zero-width but exists.
    let r = Rectangle::new(Point::new(0, 0), Size::new(4, 1));
    let parts = r.split_h_flex(&[Flex::Cell(4), Flex::Frac(1)]);
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(4, 1)));
    assert_eq!(parts[1].size.w, 0);
    assert_eq!(parts[1].pos.x, 4);
  }

  #[test]
  fn split_h_flex_too_small_for_all_cells_doesnt_panic() {
    // Pathologically small rect: should not panic, and no slice should
    // extend past the parent's right edge.
    let r = Rectangle::new(Point::new(0, 0), Size::new(0, 1));
    let parts = r.split_h_flex(&[Flex::Cell(3), Flex::Frac(1), Flex::Cell(2)]);
    assert_eq!(parts.len(), 3);
    for p in &parts {
      assert!(p.right() <= r.right());
    }
  }

  #[test]
  fn split_v_flex_basic() {
    let r = Rectangle::new(Point::new(0, 0), Size::new(5, 10));
    let parts = r.split_v_flex(&[Flex::Cell(2), Flex::Frac(1), Flex::Frac(1)]);
    assert_eq!(parts[0], Rectangle::new(Point::new(0, 0), Size::new(5, 2)));
    // Remaining 8 split 1:1 -> 4:4.
    assert_eq!(parts[1], Rectangle::new(Point::new(0, 2), Size::new(5, 4)));
    assert_eq!(parts[2], Rectangle::new(Point::new(0, 6), Size::new(5, 4)));
  }

  #[test]
  fn distribute_many() {
    use Flex::*;

    let parts = distribute(10, &[Frac(1), Frac(1), Frac(2), Frac(2)]);
    assert_eq!(parts[0], 1);
    assert_eq!(parts[1], 2);
    assert_eq!(parts[2], 3);
    assert_eq!(parts[3], 4);
  }
}
