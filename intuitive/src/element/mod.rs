mod any;
mod empty;

pub use self::{any::Any, empty::Empty};
#[allow(unused)]
use crate::components::Component;
use crate::{buffer::region::Region, error::Result};

/// A rendered [`Component`], which can be drawn onto a [`Region`].
pub trait Element {
  /// Draw the element onto the given [`Region`].
  #[allow(clippy::missing_errors_doc)]
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()>;
}
