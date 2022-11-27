use std::sync::Arc;

use parking_lot::Mutex;

use super::Component;
use crate::element::Any as AnyElement;
#[allow(unused)]
use crate::element::Element;

/// A container for functions that return an [`Element`]. This is used as a way to
/// capture a closure of a [`Component::render`] call.
pub struct Any {
  pub component: Arc<Mutex<Box<dyn Fn() -> AnyElement + Send>>>,
}

impl Any {
  /// Creates a new [`Any`].
  pub(crate) fn new<C: Component + 'static + Send>(component: C) -> Self {
    Self {
      component: Arc::new(Mutex::new(Box::new(move || component.render()))),
    }
  }

  #[must_use]
  /// Calls the inner [`Self::component`].
  pub(crate) fn render(&self) -> AnyElement {
    (self.component.lock())()
  }
}
