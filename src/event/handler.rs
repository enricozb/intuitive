use std::sync::Arc;

use super::KeyEvent;

#[derive(Clone, Default)]
pub struct Handler {
  pub handler: Option<Arc<dyn Fn(KeyEvent) + 'static + Send + Sync>>,
}

impl<F> From<F> for Handler
where
  F: Fn(KeyEvent) + 'static + Send + Sync,
{
  fn from(f: F) -> Self {
    Self {
      handler: Some(Arc::new(f)),
    }
  }
}
