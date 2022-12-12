//! Rendered components.

mod any;
mod empty;

pub use self::{
  any::{Any, Children},
  empty::Empty,
};
#[allow(unused)]
use crate::components::Component;
use crate::{buffer::region::Region, error::Result};

/// A rendered [`Component`], which can be drawn onto a [`Region`].
pub trait Element {
  /// Draw the element onto the given [`Region`].
  ///
  /// # Errors
  ///
  /// Will return an `Err` if the drawing fails.
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()>;

  // Future methods
  //
  // fn on_unmount(&self) -> Result<()> {
  //   Ok(())
  // }
  // fn on_key(&self, event: KeyEvent) -> Result<Propagation> {}
  // fn on_mouse(&self, event: MouseEvent) -> Result<Propagation> {}
  // fn on_focus(&self, event: FocusEvent) -> Result<()> {}
}
