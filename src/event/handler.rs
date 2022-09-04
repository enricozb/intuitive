use std::{ops::Deref, sync::Arc};

use super::KeyEvent;

#[derive(Clone)]
pub struct Handler {
  handler: Arc<dyn Fn(KeyEvent) + 'static + Send + Sync>,
}

impl Default for Handler {
  fn default() -> Self {
    Self { handler: Arc::new(|_| {}) }
  }
}

impl Deref for Handler {
  type Target = dyn Fn(KeyEvent) + 'static;

  fn deref(&self) -> &Self::Target {
    &*self.handler
  }
}

impl<F> From<F> for Handler
where
  F: Fn(KeyEvent) + 'static + Send + Sync,
{
  fn from(f: F) -> Self {
    Self { handler: Arc::new(f) }
  }
}
