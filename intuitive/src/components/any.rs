use std::rc::Rc;

use super::Component;
#[allow(unused)]
use crate::element::Element;
use crate::{
  element::Any as AnyElement,
  render::{ComponentID, Context},
};

/// A container for functions that return an [`Element`]. This is used as a way to
/// capture a closure of a [`Component::render`] call.
#[derive(Clone)]
pub struct Any {
  /// This component's id.
  pub id: ComponentID,

  /// The component.
  component: Rc<Box<dyn Fn(&mut Context) -> AnyElement>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub fn new<C: Component + 'static>(component_id: ComponentID, component: C) -> Self {
    Self {
      id: component_id,
      component: Rc::new(Box::new(move |context: &mut Context| component.render(context))),
    }
  }

  /// Calls the inner [`Self::component`].
  #[must_use]
  pub(crate) fn render(&self, context: &mut Context) -> AnyElement {
    AnyElement::new((self.component)(context))
  }
}
