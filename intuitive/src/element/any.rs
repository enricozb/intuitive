use std::{cell::Cell, rc::Rc};

use super::{Element, Empty};
#[allow(unused)]
use crate::draw::Region;
use crate::{error::Result, utils::array::Array};

/// A container for any type that implements [`Element`].
#[derive(Clone)]
pub struct Any {
  element: Rc<Cell<Box<dyn Element>>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub fn new<E: Element + 'static>(element: E) -> Self {
    Self {
      element: Rc::new(Cell::new(Box::new(element))),
    }
  }

  /// Swaps the inner [`Element`]s.
  pub fn swap(&self, other: &Self) {
    self.element.swap(&other.element);
  }

  /// Draws the inner [`Element`] on to a [`Region`].
  pub(crate) fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    if region.is_empty() {
      return Ok(());
    }

    let cell = &self.element;

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

/// An [`Array`] of [`Any`](Any).
pub type Children<const N: usize> = Array<N, Any>;
