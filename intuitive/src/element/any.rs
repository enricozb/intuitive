use std::{
  cell::Cell,
  fmt::{Debug, Formatter},
  sync::Arc,
};

use parking_lot::Mutex;

use super::{Element, Empty};
#[allow(unused)]
use crate::buffer::region::Region;
use crate::{error::Result, utils::array::Array};

/// A container for any type that implements [`Element`].
#[derive(Clone)]
pub struct Any {
  element: Arc<Mutex<Cell<Box<dyn Element + Send>>>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub fn new<E: Element + 'static + Send>(element: E) -> Self {
    Self {
      element: Arc::new(Mutex::new(Cell::new(Box::new(element)))),
    }
  }

  /// Swaps the inner [`Element`]s.
  pub fn swap(&self, other: &Self) {
    self.element.lock().swap(&other.element.lock());
  }

  /// Draws the inner [`Element`] on to a [`Region`].
  pub(crate) fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    if region.is_empty() {
      return Ok(());
    }

    let cell = self.element.lock();

    let element = cell.replace(Box::new(Empty));
    element.draw(region)?;
    cell.set(element);

    Ok(())
  }
}

impl Element for Any {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    self.draw(region)
  }
}

impl Default for Any {
  fn default() -> Self {
    Self::new(Empty)
  }
}

/// An [`Array`] of [`AnyElement`][struct@Any].
pub type Children<const N: usize> = Array<N, Any>;
