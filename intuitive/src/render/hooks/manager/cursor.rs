#[allow(unused)]
use crate::render::manager::Manager as RenderManager;
use crate::render::{
  hooks::{error::Result, Hook, Hooks},
  ComponentID,
};

/// A cursor for reading memoized hook values during [`RenderManager::render`] calls.
pub(crate) struct Cursor {
  pub component_id: ComponentID,

  hooks: Hooks,
  idx: usize,
}

impl Cursor {
  /// Creates a new [`Cursor`].
  pub fn new(component_id: ComponentID, hooks: Hooks) -> Self {
    Self { component_id, hooks, idx: 0 }
  }

  /// Calls [`Hooks::seal`] as appropriate.
  pub fn done(mut self) -> Result<Hooks> {
    if !self.hooks.sealed {
      self.hooks.seal()?;
    }

    Ok(self.hooks)
  }

  /// Returns the next hook value, creating it using `f` if necessary.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if type `T` is invalid, or if the cursor's index is invalid.
  pub fn next<F, T>(&mut self, f: F) -> Result<T>
  where
    F: FnOnce(ComponentID) -> Hook,
    T: 'static + Clone,
  {
    let value = if self.hooks.sealed {
      self.hooks.get(self.idx)?.downcast_ref()?
    } else {
      let hook = f(self.component_id);
      let value = hook.downcast_ref()?;

      self.hooks.push(hook)?;

      value
    };

    self.idx += 1;

    Ok(value)
  }
}
