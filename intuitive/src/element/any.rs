use std::{cell::Cell, sync::Arc};

use parking_lot::Mutex;

use super::{Element, Empty};
#[allow(unused)]
use crate::buffer::region::Region;

/// A container for any type that implements [`Element`].
#[derive(Clone)]
pub struct Any {
  element: Arc<Mutex<Cell<Box<dyn Element + Send>>>>,
}

impl Any {
  /// Creates a new [`Any`].
  fn new<E: Element + 'static + Send>(element: E) -> Self {
    Self {
      element: Arc::new(Mutex::new(Cell::new(Box::new(element)))),
    }
  }

  /// Swaps the inner [`Element`]s.
  pub fn swap(&self, other: &Self) {
    self.element.lock().swap(&other.element.lock());
  }

  /// Draws the inner [`Element`] on to a [`Region`].
  pub(crate) fn draw(&self) {
    let cell = self.element.lock();

    let element = cell.replace(Box::new(Empty));
    element.draw();
    cell.set(element);
  }
}
