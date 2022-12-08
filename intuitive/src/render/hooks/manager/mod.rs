mod cursor;
use std::collections::HashMap;

use self::cursor::Cursor;
use super::error::{Error, Result};
use crate::render::{
  hooks::{Hook, Hooks},
  ComponentID,
};

/// Manages hooks.
pub struct Manager {
  /// A stack of [`Cursor`]s that keep track of what hooks have been used in a component.
  cursors: Vec<Cursor>,

  /// Maps [`ComponentID`]s to a component's [`Hooks`].
  hooks: HashMap<ComponentID, Hooks>,
}

impl Manager {
  /// Creates a new [`Manager`].
  #[must_use]
  pub fn new() -> Self {
    Self {
      cursors: Vec::new(),
      hooks: HashMap::new(),
    }
  }

  /// Pushes a new [`Cursor`] to the stack.
  pub(crate) fn push_cursor(&mut self, component_id: ComponentID) {
    let cursor = match self.hooks.remove(&component_id) {
      Some(hooks) => Cursor::new(component_id, hooks),
      None => Cursor::new(component_id, Hooks::new()),
    };

    self.cursors.push(cursor);
  }

  /// Pops a [`Cursor`] from the stack.
  pub(crate) fn pop_cursor(&mut self) {
    let cursor = self.cursors.pop().ok_or(Error::NoCursor).expect("pop");
    let component_id = cursor.component_id;
    let hooks = cursor.done().expect("Cursor::done");

    self.hooks.insert(component_id, hooks);
  }

  /// Returns the inner value of the current [`Hook`], constructing it with `f` if necessary.
  ///
  /// The parameter `f` is not generic because `use_hook` is often used with a turbofish, and it
  /// would be difficult (impossible?) to specify the type of a closure.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if there is no [`Cursor`] at the top of the stack, or if
  /// [`Cursor::next`] returns an `Err`.
  pub fn use_hook<T>(&mut self, f: impl FnOnce(ComponentID) -> Hook) -> Result<T>
  where
    T: 'static + Clone,
  {
    self.cursors.last_mut().ok_or(Error::NoCursor)?.next(f)
  }

  /// Returns the current [`ComponentID`] at the top of the [`Self::cursors`] stack.
  pub fn current_component_id(&self) -> Option<ComponentID> {
    self.cursors.last().map(|cursor| cursor.component_id)
  }

  /// Unmounts the component, deinitializing all of its hooks.
  pub fn unmount(&mut self, component_id: ComponentID) {
    if let Some(hooks) = self.hooks.remove(&component_id) {
      hooks.deinit();
    }
  }
}
