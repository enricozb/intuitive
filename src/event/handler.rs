use std::sync::Arc;

use super::KeyEvent;

#[derive(Clone, Default)]
pub struct Handler {
  handler: Option<Arc<dyn Fn(KeyEvent) + 'static + Send + Sync>>,
}

impl Handler {
  pub fn handle(&self, event: KeyEvent) {
    self.handle_or(event, |_| {})
  }

  pub fn handle_or<F>(&self, event: KeyEvent, alternative_handler: F)
  where
    F: FnOnce(KeyEvent),
  {
    if let Some(handler) = &self.handler {
      handler(event)
    } else {
      alternative_handler(event)
    }
  }
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
