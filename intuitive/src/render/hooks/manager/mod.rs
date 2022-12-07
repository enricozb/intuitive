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
  /// A stack of [`Cursor`]s that are pushed/popped before/after [`with`].
  static ref CURSORS: Mutex<Vec<Cursor>> = Mutex::new(Vec::new());

  /// Maps [`ComponentID`]s to a component's [`Hooks`].
  static ref HOOKS: Mutex<HashMap<ComponentID, Hooks>> = Mutex::new(HashMap::new());
}

/// Run a function `f` with `component_id` at the top of the [`CURSORS`] stack.
pub fn with<F, T>(component_id: ComponentID, f: F) -> T
where
  F: FnOnce() -> T,
{
  let cursor = match HOOKS.lock().remove(&component_id) {
    Some(hooks) => Cursor::new(component_id, hooks),
    None => Cursor::new(component_id, Hooks::new()),
  };

  CURSORS.lock().push(cursor);

  let ret = f();

  let hooks = CURSORS
    .lock()
    .pop()
    .ok_or(Error::NoCursor)
    .expect("with: pop")
    .done()
    .expect("Cursor::done");

  HOOKS.lock().insert(component_id, hooks);

  ret
}

/// Returns the inner value of the current [`Hook`], constructing it with `f` if necessary.
///
/// The parameter `f` is not generic because `use_hook` is often used with a turbofish, and it
/// would be difficult (impossible?) to specify the type of a closure.
pub fn use_hook<T>(f: impl FnOnce(ComponentID) -> Hook) -> Result<T>
where
  T: 'static + Send + Sync + Clone,
{
  CURSORS.lock().last_mut().ok_or(Error::NoCursor)?.next(f)
}

/// Returns the current [`ComponentID`] at the top of the [`CURSORS`] stack.
pub fn current_component_id() -> Option<ComponentID> {
  CURSORS.lock().last_mut().map(|cursor| cursor.component_id)
}

/// Unmounts the component, deinitializing all of its hooks.
pub fn unmount(component_id: ComponentID) {
  if let Some(hooks) = HOOKS.lock().remove(&component_id) {
    hooks.deinit();
  }
}
