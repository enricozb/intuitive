use std::any::{self, Any};

use super::ComponentID;

pub mod error;
pub mod manager;

use self::error::{Error, Result};

/// A hook initializer.
trait Initializer<T>: FnOnce(&ComponentID) -> T {}

/// Memoized return values of hook [`Initializer`] calls.
struct Memos(Vec<Box<dyn Any + Send + Sync>>);

impl Memos {
  /// Creates a new [`Memos`].
  pub fn new() -> Self {
    Self(Vec::new())
  }

  /// Retrieves a value at a specific index, `idx`, downcasting it to `T`.
  pub fn get<T: 'static + Clone>(&self, idx: usize) -> Result<T> {
    Ok(
      self
        .0
        .get(idx)
        .ok_or(Error::InvalidIdx(idx))?
        .downcast_ref::<T>()
        .ok_or_else(|| Error::InvalidType(any::type_name::<T>()))?
        .clone(),
    )
  }

  pub fn push<T: 'static + Send + Sync>(&mut self, val: T) {
    self.0.push(Box::new(val));
  }
}