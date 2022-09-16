use std::sync::Arc;

/// Whether to propagate the event to the next handler.
pub enum Propagate {
  Next,
  Stop,
}

/// A generic handler for mouse and keyboard events.
pub struct Handler<T> {
  handler: Arc<dyn Fn(T) -> Propagate + 'static + Send + Sync>,
}

impl<T> Default for Handler<T> {
  fn default() -> Self {
    Self {
      handler: Arc::new(|_| Propagate::Next),
    }
  }
}

impl<T> Clone for Handler<T> {
  fn clone(&self) -> Self {
    Self {
      handler: self.handler.clone(),
    }
  }
}

impl<T: 'static + Copy> Handler<T> {
  /// Call the handler on the event.
  pub fn handle(&self, event: T) {
    self.handle_or(event, |_| {});
  }

  /// Call the handler on the event, defaulting to the alternative_handler.
  pub fn handle_or<F, R>(&self, event: T, alternative_handler: F)
  where
    F: FnOnce(T) -> R,
  {
    match (self.handler)(event) {
      Propagate::Next => drop(alternative_handler(event)),
      Propagate::Stop => (),
    }
  }

  /// Create a new handler that propagates to `next_handler`.
  ///
  /// Propagation only occurs if this handler returns `Propagate::Next`.
  pub fn then<F>(&self, next_handler: F) -> Self
  where
    F: Fn(T) -> Propagate + 'static + Send + Sync,
  {
    let handler = self.handler.clone();

    Handler::from(move |event| match handler(event) {
      Propagate::Next => next_handler(event),
      Propagate::Stop => Propagate::Stop,
    })
  }
}

impl<F, T> From<F> for Handler<T>
where
  F: Fn(T) -> Propagate + 'static + Send + Sync,
{
  fn from(f: F) -> Self {
    Self { handler: Arc::new(f) }
  }
}

impl<T> From<&Handler<T>> for Handler<T> {
  fn from(handler: &Handler<T>) -> Self {
    handler.clone()
  }
}
