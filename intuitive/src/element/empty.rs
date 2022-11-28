use super::Element;
use crate::{buffer::region::Region, error::Result};

/// The empty [`Element`]. It draws nothing.
pub struct Empty;

impl Element for Empty {
  fn draw<'a>(&self, _region: Region<'a>) -> Result<()> {
    Ok(())
  }
}
