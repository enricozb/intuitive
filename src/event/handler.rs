use std::{ops::Deref, rc::Rc};

use super::KeyEvent;

#[derive(Clone)]
pub struct Key {
  handler: Rc<dyn Fn(KeyEvent) + 'static>,
}

impl Default for Key {
  fn default() -> Self {
    Self { handler: Rc::new(|_| {}) }
  }
}

impl Deref for Key {
  type Target = dyn Fn(KeyEvent) + 'static;

  fn deref(&self) -> &Self::Target {
    &*self.handler
  }
}

impl<F> From<F> for Key
where
  F: Fn(KeyEvent) + 'static,
{
  fn from(f: F) -> Self {
    Self { handler: Rc::new(f) }
  }
}
