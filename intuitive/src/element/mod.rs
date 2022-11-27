mod any;
mod empty;

pub use self::{any::Any, empty::Empty};
#[allow(unused)]
use crate::buffer::Buffer;
#[allow(unused)]
use crate::components::Component;

/// A rendered [`Component`], which can be drawn onto a [`Region`] of a [`Buffer`].
pub trait Element {
  fn draw(&self);
}
