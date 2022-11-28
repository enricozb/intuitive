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
  /// Will return `Err` if the desired region exceeds the current region's bounds.
  pub fn narrow(&'a mut self, position: Position, size: Size) -> Result<Self> {
    if position.x + size.width > self.size.width || position.y + size.height > self.size.height {
      return Err(Error::RegionOutOfBounds);
    }

    Ok(Self {
      position: self.position + position,
      size,
      buffer: self.buffer,
    })
  }

  #[must_use]
  /// Returns the [`Size`] of the [`Region`].
  pub fn size(&self) -> Size {
    self.size
  }
}

impl<'a> Draw for Region<'a> {
  fn set_option_cell(&mut self, position: Position, cell: Option<Cell>) {
    self.buffer.set_option_cell(position + self.position, cell);
  }
}
