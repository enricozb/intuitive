use super::Element;
use crate::{buffer::region::Region, error::Result};

/// The empty [`Element`]. It draws nothing.
pub struct Empty;

impl Element for Empty {
  fn draw(&self, _region: Region) -> Result<()> {
    Ok(())
  }
}
