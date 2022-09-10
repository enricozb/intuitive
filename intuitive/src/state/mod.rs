//! Primitives for handling state.

mod hook;
mod manager;

use std::sync::Arc;

use parking_lot::Mutex;

pub(crate) use self::hook::render_done;
pub use self::hook::use_state;
use crate::event;

/// A struct that triggers a re-render upon mutation.
///
/// `State`'s have interior mutability, and can be cloned. `State`s cloned
/// from one another share the inner reference to a `T`, and therefore mutating one of
/// them will be reflected across all of the cloned states. For example,
/// ```rust
/// let count = use_state(|| 0);
///
/// let other_count = count.clone();
/// other_count.set(1);
///
/// // both `count` and `other_count` are `1`
/// assert_eq!(count.get(), other_count.get());
/// ```
///
/// This is useful when receiving a `State` as a parameter from a parent component,
/// as it must be cloned, and then may be mutated by both the child and parent components.
#[derive(Default)]
pub struct State<T> {
  inner: Arc<Mutex<T>>,
}

impl<T> State<T> {
  pub(crate) fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  /// Sets a new value for the state and triggers a re-render.
  pub fn set(&self, new: T) {
    let mut inner = self.inner.lock();
    *inner = new;

    event::re_render().expect("re_render");
  }

  /// Calls a function on the inner value and returns its result.
  /// Does not trigger a re-render.
  pub fn inspect<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&self.inner.lock())
  }

  /// Calls a function on a mutable reference of the inner value and triggers a re-render.
  pub fn mutate<F, R>(&self, f: F)
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut inner = self.inner.lock();
    drop(f(&mut inner));

    event::re_render().expect("re_render");
  }

  /// Calls a function on the inner value, replaces it with the result, and triggers a re-render.
  pub fn update<F>(&self, f: F)
  where
    F: FnOnce(&T) -> T,
  {
    let mut inner = self.inner.lock();
    *inner = f(&inner);

    event::re_render().expect("re_render");
  }
}

impl<T: Clone> State<T> {
  /// Returns a clone of the `State<T>`'s inner value.
  pub fn get(&self) -> T {
    self.inner.lock().clone()
  }
}

impl<T> Clone for State<T> {
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}
