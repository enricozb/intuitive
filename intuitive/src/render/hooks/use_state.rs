use std::sync::Arc;

use parking_lot::Mutex;

use crate::render::hooks::manager;

/// A container for a `T` which causes re-renders when mutated.
///
/// This is typically not created manually, and is instead created through [`use_state`].
pub struct State<T> {
  inner: Arc<Mutex<T>>,
}

impl<T> State<T> {
  /// Creates a new [`State<T>`].
  fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  /// Retrieves [`State<T>`].
  pub fn get(&self) -> T
  where
    T: Clone,
  {
    // TODO(enricozb): implement signals:
    //   This should check the current `ComponentID` in use, and associate this state with that `ComponentID`.
    //   Then, on any changes to this `State<T>`, rerenders should be triggered for those `ComponentID`s.

    self.inner.lock().clone()
  }
}

impl<T> Clone for State<T> {
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

/// A hook to add state to a component.
///
/// Returns a [`State<T>`] initialized with the provided `initializer`. The `initializer` is only called once,
/// when the `use_state` hook is first called, and a memoized value is used in future calls.
///
/// This is inspired by React's [`useState`] hook.
///
/// [`useState`]: https://reactjs.org/docs/hooks-state.html
pub fn use_state<F, T>(initializer: F) -> State<T>
where
  F: FnOnce() -> T,
  T: 'static + Send,
{
  manager::use_hook(|_| State::new(initializer())).expect("use_state: use_hook")
}
