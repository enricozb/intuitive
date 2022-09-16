use std::sync::Arc;

/// A generic handler for mouse and keyboard events.
pub struct Handler<T> {
  handler: Option<Arc<dyn Fn(T) + 'static + Send + Sync>>,
}

impl<T> Default for Handler<T> {
  fn default() -> Self {
    Self { handler: None }
  }
}

impl<T> Clone for Handler<T> {
  fn clone(&self) -> Self {
    Self {
      handler: self.handler.clone(),
    }
  }
}

impl<T> Handler<T> {
  /// Call the handler on the event.
  pub fn handle(&self, event: T) {
    self.handle_or(event, |_| {});
  }

  /// Call the handler on the event, defaulting to the alternative_handler.
  pub fn handle_or<F>(&self, event: T, alternative_handler: F)
  where
    F: FnOnce(T),
  {
    if let Some(handler) = &self.handler {
      handler(event);
    } else {
      alternative_handler(event);
    }
  }
}

impl<F, T> From<F> for Handler<T>
where
  F: Fn(T) + 'static + Send + Sync,
{
  fn from(f: F) -> Self {
    Self {
      handler: Some(Arc::new(f)),
    }
  }
}

impl<T> From<&Handler<T>> for Handler<T> {
  fn from(handler: &Handler<T>) -> Self {
    handler.clone()
  }
}
