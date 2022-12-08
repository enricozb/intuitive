use std::rc::Rc;

use super::Component;
#[allow(unused)]
use crate::element::Element;
use crate::{element::Any as AnyElement, render::Manager as RenderManager};

/// A container for functions that return an [`Element`]. This is used as a way to
/// capture a closure of a [`Component::render`] call.
#[derive(Clone)]
pub(crate) struct Any {
  pub component: Rc<Box<dyn Fn(&mut RenderManager) -> AnyElement>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub(crate) fn new<C: Component + 'static>(component: C) -> Self {
    Self {
      component: Rc::new(Box::new(move |render: &mut RenderManager| component.render(render))),
    }
  }

  #[must_use]
  /// Calls the inner [`Self::component`].
  pub(crate) fn render(&self, render: &mut RenderManager) -> AnyElement {
    (self.component)(render)
  }
}
