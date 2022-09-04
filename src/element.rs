use std::{ops::Deref, sync::Arc};

use crate::{
  components::{Component, Empty},
  event::KeyEvent,
  terminal::{Frame, Rect},
};

pub struct Any(Arc<dyn Element>);

impl Any {
  pub fn new<C: Element + 'static>(element: C) -> Self {
    Self(Arc::new(element))
  }
}

impl<'a> Default for Any {
  fn default() -> Self {
    Empty.render()
  }
}

impl Deref for Any {
  type Target = Arc<dyn Element>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<C: Component> From<C> for Any {
  fn from(component: C) -> Self {
    component.render()
  }
}

pub trait Element {
  fn on_key(&self, _event: KeyEvent) {}
  fn draw(&self, _rect: Rect, _frame: &mut Frame) {}
}
