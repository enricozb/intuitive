use std::{ops::Deref, rc::Rc};

use super::{Component, Empty};

#[derive(Clone)]
pub struct Any(Rc<dyn Component>);

impl Any {
  fn new<C: Component + 'static>(component: C) -> Self {
    Any(Rc::new(component))
  }
}

impl Default for Any {
  fn default() -> Self {
    Empty.into()
  }
}

impl Deref for Any {
  type Target = Rc<dyn Component>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<C: Component + 'static> From<C> for Any {
  fn from(component: C) -> Self {
    Self::new(component)
  }
}
