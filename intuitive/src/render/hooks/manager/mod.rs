mod cursor;
use std::collections::HashMap;

use lazy_static::lazy_static;
use parking_lot::Mutex;

use self::cursor::Cursor;
use super::error::{Error, Result};
use crate::render::{
  hooks::{Hook, Hooks},
  ComponentID,
};

lazy_static! {
  /// The global hook [`Manager`].
  static ref MANAGER: Manager = Manager::new();
}

/// Calls [`Manager::with`] for the global [`Manager`].
#[allow(rustdoc::private_intra_doc_links)]
pub fn with<F, T>(id: ComponentID, f: F) -> T
where
  F: FnOnce() -> T,
{
  MANAGER.with(id, f)
}

/// A primitive hook used to implement higher-level hooks.
#[allow(rustdoc::private_intra_doc_links)]
pub fn use_hook<T>(f: impl FnOnce(ComponentID) -> Hook) -> Result<T>
where
  T: 'static + Send + Sync + Clone,
{
  MANAGER.use_hook(f)
}

/// Manages [`use_hook`] calls across renders.
///
/// [`Manager`]s have interior mutability, so they can be [`Sync`].
pub struct Manager {
  /// A stack of [`Cursor`]s that are pushed/popped before/after rendering.
  cursors: Mutex<Vec<Cursor>>,
  /// Maps [`ComponentID`]s to a component's [`Hooks`].
  hooks: Mutex<HashMap<ComponentID, Hooks>>,
}

impl Manager {
  /// Creates a new [`Manager`].
  fn new() -> Self {
    Self {
      cursors: Mutex::new(Vec::new()),
      hooks: Mutex::new(HashMap::new()),
    }
  }

  /// Calls `f` with a [`Cursor`] for the given [`ComponentID`] at the top of the [`Self::cursors`] stack.
  fn with<F, T>(&self, component_id: ComponentID, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    let cursor = match self.hooks.lock().remove(&component_id) {
      Some(hooks) => Cursor::new(component_id, hooks),
      None => Cursor::new(component_id, Hooks::new()),
    };

    self.cursors.lock().push(cursor);

    let ret = f();

    let hooks = self
      .cursors
      .lock()
      .pop()
      .ok_or(Error::NoCursor)
      .expect("with: pop")
      .done()
      .expect("Cursor::done");

    self.hooks.lock().insert(component_id, hooks);

    ret
  }

  /// Returns the inner value of the current [`Hook`], constructing it with `f` if necessary.
  fn use_hook<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(ComponentID) -> Hook,
    T: 'static + Send + Sync + Clone,
  {
    let mut cursors = self.cursors.lock();

    cursors.last_mut().ok_or(Error::NoCursor)?.next(f)
  }
}
