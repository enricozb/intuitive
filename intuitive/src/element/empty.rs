use super::Element;
use crate::{
  draw::{Context, Region},
  error::Result,
};

/// The empty [`Element`]. It draws nothing.
pub struct Empty;

impl Element for Empty {
  fn draw<'a>(&self, _context: &mut Context, _region: &'a mut Region<'a>) -> Result<()> {
    Ok(())
  }
}
