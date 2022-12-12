pub mod error;
mod use_effect;
mod use_effect_with_deps;
mod use_state;

use std::any::{self, Any};

use self::error::{Error, Result};
pub use self::{
  use_effect::{Cleanup, UseEffect},
  use_effect_with_deps::UseEffectWithDeps,
  use_state::{State, UseState},
};

/// A dynamically-typed hook return value, along with a deinitialization function for unmounting.
pub struct Hook {
  /// The inner value of the hook.
  value: Box<dyn Any>,

  /// Any deinitialization needed for whne this hook's parent component is unmounted.
  deinit: Option<Box<dyn FnOnce()>>,
}

impl Default for Hook {
  fn default() -> Self {
    Self {
      value: Box::new(()),
      deinit: None,
    }
  }
}

impl Hook {
  /// Creates a new [`Hook`].
  #[must_use]
  pub fn new<T, F>(value: T, deinit: F) -> Self
  where
    T: 'static,
    F: 'static + FnOnce(),
  {
    Self {
      value: Box::new(value),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Creates a new [`Hook`] with only a value, and no deinitialization function.
  pub fn from_value<T>(value: T) -> Self
  where
    T: 'static,
  {
    Self {
      value: Box::new(value),
      deinit: None,
    }
  }

  /// Creates a new [`Hook`] with only a deinitialization function, and unit value.
  pub fn from_deinit<F>(deinit: F) -> Self
  where
    F: 'static + FnOnce(),
  {
    Self {
      value: Box::new(()),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Calls the `deinit` function
  pub fn deinit(self) {
    if let Some(deinit) = self.deinit {
      deinit();
    }
  }

  /// Calls [`Any.downcast_ref`] on the [`Hook`]s inner value.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if the hook's value can't be cast to `T`.
  pub fn downcast_ref<T: 'static + Clone>(&self) -> Result<T> {
    Ok(self.value.downcast_ref::<T>().ok_or(Error::InvalidType(any::type_name::<T>()))?.clone())
  }
}
