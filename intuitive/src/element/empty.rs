use super::Element;
use crate::{draw::Region, error::Result};

/// The empty [`Element`]. It draws nothing.
pub struct Empty;

impl Element for Empty {
  fn draw<'a>(&self, _region: &'a mut Region<'a>) -> Result<()> {
    Ok(())
  }
}
