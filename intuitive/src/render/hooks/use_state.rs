use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
  error::Result,
  event,
  render::{hooks::Hook, providers::Hooks, ComponentID},
};

/// A container for a `T` which causes re-renders when mutated, returned by [`UseState::use_state`].
///
/// `State` is created through [`UseState::use_state`].
pub struct State<T> {
  component_id: ComponentID,
  inner: Arc<Mutex<T>>,
}

impl<T> State<T> {
  /// Creates a new [`State`].
  fn new(component_id: ComponentID, inner: T) -> Self {
    Self {
      component_id,
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  /// Triggers a re-render for the component that this [`State`] was defined in.
  fn rerender(&self) -> Result<()> {
    event::rerender(self.component_id)
  }

  /// Returns a clone of the inner `T`.
  #[must_use]
  pub fn get(&self) -> T
  where
    T: Clone,
  {
    // TODO(enricozb): implement signals:
    //   This should check the current `ComponentID` in use, and associate this state with that `ComponentID`.
    //   Then, on any changes to this `State`, re-renders should be triggered for those `ComponentID`s.

    self.inner.lock().clone()
  }

  /// Sets the inner value of this [`State`] and enqueues a re-render.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if enqueueing the re-render fails.
  pub fn set(&self, new: T) -> Result<()> {
    let mut inner = self.inner.lock();
    *inner = new;

    self.rerender()
  }

  /// Mutates the inner value of this [`State`] using the provided function, returning its return value,
  /// and enqueues a re-render.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if enqueueing the re-render fails.
  pub fn mutate<F, R>(&self, f: F) -> Result<R>
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut inner = self.inner.lock();
    let ret = f(&mut inner);

    self.rerender()?;

    Ok(ret)
  }

  /// Updates the inner value of this [`State`] with the return value of the provided function
  /// and enqueues a re-render.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if enqueueing the re-render fails.
  pub fn update<F>(&self, f: F) -> Result<()>
  where
    F: FnOnce(&T) -> T,
  {
    let mut inner = self.inner.lock();
    *inner = f(&inner);

    self.rerender()
  }
}

impl<T> Clone for State<T> {
  fn clone(&self) -> Self {
    Self {
      component_id: self.component_id,
      inner: self.inner.clone(),
    }
  }
}

impl<T: 'static> From<State<T>> for Hook {
  fn from(state: State<T>) -> Self {
    Self::from_value(state)
  }
}

/// A hook to add state to a component.
pub trait UseState {
  /// Returns a [`State`] initialized with the provided `initializer`. The `initializer` is only called once,
  /// when the [`UseState::use_state`] hook is first called, and a memoized value is used in future calls.
  ///
  /// When a [`State`] is mutated through its methods, it will trigger a re-render of the component that it
  /// was first defined in.
  ///
  /// This is inspired by React's [`useState`] hook.
  ///
  /// [`useState`]: https://reactjs.org/docs/hooks-state.html
  fn use_state<F, T>(&mut self, initializer: F) -> State<T>
  where
    F: FnOnce() -> T,
    T: 'static;
}

impl UseState for Hooks {
  fn use_state<F, T>(&mut self, initializer: F) -> State<T>
  where
    F: FnOnce() -> T,
    T: 'static,
  {
    self
      .use_hook(|component_id| State::new(component_id, initializer()).into())
      .expect("use_state: use_hook")
  }
}