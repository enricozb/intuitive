pub(crate) mod use_effect;
pub(crate) mod use_state;

pub(crate) mod error;
pub(crate) mod manager;

use std::any::{self, Any};

use self::error::{Error, Result};
pub use self::{
  manager::use_hook,
  use_effect::use_effect,
  use_state::{use_state, State},
};

/// A dynamically-typed hook return value, along with a deinitialization function for unmounting.
pub struct Hook {
  /// The inner value of the hook.
  value: Box<dyn Any + Send + Sync>,

  /// Any deinitialization needed for whne this hook's parent component is unmounted.
  deinit: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl Hook {
  /// Creates a new [`Hook`].
  pub fn new<T, F>(value: T, deinit: F) -> Self
  where
    T: 'static + Send + Sync,
    F: 'static + FnOnce() + Send + Sync,
  {
    Self {
      value: Box::new(value),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Creates a new [`Hook`] with only a value, and no deinitialization function.
  pub fn from_value<T>(value: T) -> Self
  where
    T: 'static + Send + Sync,
  {
    Self {
      value: Box::new(value),
      deinit: None,
    }
  }

  /// Creates a new [`Hook`] with only a deinitialization function, and unit value.
  pub fn from_deinit<F>(deinit: F) -> Self
  where
    F: 'static + FnOnce() + Send + Sync,
  {
    Self {
      value: Box::new(()),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Calls the `deinit` function
  pub fn deinit(self) {
    if let Some(deinit) = self.deinit {
      deinit()
    }
  }

  /// Calls [`Any.downcast_ref`] on the [`Hook`]s inner value.
  pub fn downcast_ref<T: 'static + Clone>(&self) -> Result<T> {
    Ok(
      self
        .value
        .downcast_ref::<T>()
        .ok_or(Error::InvalidType(any::type_name::<T>()))?
        .clone(),
    )
  }
}

/// A vector of hooks, that is initially open for writing, and then is sealed after the first render of a component.
pub(crate) struct Hooks {
  hooks: Vec<Hook>,
  sealed: bool,
}

impl Hooks {
  /// Creates a new [`Self`].
  pub fn new() -> Self {
    Self {
      hooks: Vec::new(),
      sealed: false,
    }
  }

  /// Gets a hook at an index.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if the index is invalid, or if [`Self::sealed`] is `false`.
  pub fn get(&self, idx: usize) -> Result<&Hook> {
    if !self.sealed {
      return Err(Error::NotSealed);
    }

    self.hooks.get(idx).ok_or(Error::InvalidIdx(idx))
  }

  /// Pushes a new [`Hook`].
  ///
  /// # Errors
  ///
  /// Will return an `Err` if [`Self::seal`] has already been called.
  pub fn push(&mut self, hook: Hook) -> Result<()> {
    if self.sealed {
      return Err(Error::Sealed);
    }

    self.hooks.push(hook);

    Ok(())
  }

  /// Seals itself, erroring on any future calls to [`Self::push`].
  ///
  /// # Errors
  ///
  /// Will return an `Err` if [`Self::seal`] has already been called.
  pub fn seal(&mut self) -> Result<()> {
    if self.sealed {
      return Err(Error::AlreadySealed);
    }

    self.sealed = true;

    Ok(())
  }
}
