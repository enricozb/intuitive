use std::rc::Rc;

use super::Component;
#[allow(unused)]
use crate::element::Element;
use crate::{
  element::Any as AnyElement,
  render::{manager::Manager as RenderManager, ComponentID},
};

/// A container for functions that return an [`Element`]. This is used as a way to
/// capture a closure of a [`Component::render`] call.
#[derive(Clone)]
pub struct Any {
  /// This component's id.
  pub id: ComponentID,

  /// The component.
  component: Rc<Box<dyn Fn(&mut RenderManager) -> AnyElement>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub fn new<C: Component + 'static>(component_id: ComponentID, component: C) -> Self {
    Self {
      id: component_id,
      component: Rc::new(Box::new(move |render: &mut RenderManager| component.render(render))),
    }
  }

  /// Calls the inner [`Self::component`].
  #[must_use]
  pub(crate) fn render(&self, render: &mut RenderManager) -> AnyElement {
    (self.component)(render)
  }
}
