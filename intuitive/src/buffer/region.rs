use super::{draw::Draw, Buffer, Cell};
use crate::{
  error::{Error, Result},
  utils::geometry::{Position, Size},
};

/// A region within a [`Buffer`].
///
/// [`Region`]s have a specific [`Position`] and [`Size`] within their parent [`Buffer`].
pub struct Region<'a> {
  position: Position,
  size: Size,
  buffer: &'a mut Buffer,
}

/// Creates a [`Region`] that encompasses the entire [`Buffer`].
impl<'a> From<&'a mut Buffer> for Region<'a> {
  fn from(buffer: &'a mut Buffer) -> Self {
    Self {
      position: Position::default(),
      size: buffer.size(),
      buffer,
    }
  }
}

impl<'a> Region<'a> {
  /// Narrow a [`Region`] using a relative [`Position`] and [`Size`].
  ///
  /// # Errors
  ///
  /// Will return an `Err` if the desired region exceeds the current region's bounds.
  pub fn narrow<'b>(&'b mut self, position: Position, size: Size) -> Result<Region<'b>>
  where
    'a: 'b,
  {
    if position.x + size.width > self.size.width || position.y + size.height > self.size.height {
      return Err(Error::RegionOutOfBounds);
    }

    Ok(Region {
      position: self.position + position,
      size,
      buffer: self.buffer,
    })
  }

  /// Returns whether a [`Position`] is within the bounds of this [`Region`].
  #[must_use]
  pub fn in_bounds(&self, position: Position) -> bool {
    position.x < self.size.width && position.y < self.size.height
  }

  /// Returns the [`Size`] of the [`Region`].
  #[must_use]
  pub fn size(&self) -> Size {
    self.size
  }

  /// Returns if the region is empty, which returns true if either [`Self::size`] dimention is `0`.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.size.width == 0 || self.size.height == 0
  }
}

impl<'a> Draw for Region<'a> {
  fn set_option_cell(&mut self, position: Position, cell: Option<Cell>) -> bool {
    let in_bounds = self.in_bounds(position);

    if in_bounds {
      self.buffer.set_option_cell(position + self.position, cell);
    }

    in_bounds
  }

  fn size(&self) -> Size {
    self.size
  }
}
