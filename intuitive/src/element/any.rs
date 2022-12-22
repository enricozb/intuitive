use std::{cell::Cell, rc::Rc};

use super::{Element, Empty};
#[allow(unused)]
use crate::draw::{Context, Region};
use crate::{error::Result, render::ComponentID, utils::array::Array};

/// A container for any type that implements [`Element`].
#[derive(Clone)]
pub struct Any {
  /// The ID of the component that rendered this element.
  ///
  /// If `None`, this element won't be drawn.
  pub(crate) id: Option<ComponentID>,

  element: Rc<Cell<Box<dyn Element>>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub fn new<E: Element + 'static>(id: ComponentID, element: E) -> Self {
    Self {
      id: Some(id),
      element: Rc::new(Cell::new(Box::new(element))),
    }
  }

  /// Swaps the inner [`Element`]s.
  pub fn swap(&self, other: &Self) {
    self.element.swap(&other.element);
  }

  /// Draws the inner [`Element`] on to a [`Region`].
  pub(crate) fn draw<'a>(&self, context: &mut Context, region: &'a mut Region<'a>) -> Result<()> {
    if region.is_empty() {
      return Ok(());
    }

    let cell = &self.element;

    let element = cell.replace(Box::new(Empty));
    element.draw(context, region)?;
    cell.set(element);

    Ok(())
  }
}

// TODO(enricozb): why do we need this?
impl Element for Any {
  fn draw<'a>(&self, context: &mut Context, region: &'a mut Region<'a>) -> Result<()> {
    self.draw(context, region)
  }
}

impl Default for Any {
  fn default() -> Self {
    Self {
      id: None,
      element: Rc::new(Cell::new(Box::new(Empty))),
    }
  }
}

/// An [`Array`] of [`Any`](Any).
pub type Children<const N: usize> = Array<N, Any>;
