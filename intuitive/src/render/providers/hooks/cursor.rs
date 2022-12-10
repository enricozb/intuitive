use crate::render::{
  hooks::{
    error::{Error, Result},
    Hook,
  },
  ComponentID,
};

/// Tracks hook usage within a render.
pub struct Cursor {
  pub component_id: ComponentID,

  hooks: Vec<Hook>,
  index: usize,

  writing: bool,
}

impl Cursor {
  /// Creates a reading [`Cursor`].
  pub fn read(component_id: ComponentID, hooks: Vec<Hook>) -> Self {
    Self {
      component_id,
      hooks,
      index: 0,

      writing: false,
    }
  }

  /// Creates a writing [`Cursor`].
  pub fn write(component_id: ComponentID) -> Self {
    Self {
      component_id,
      hooks: Vec::new(),
      index: 0,

      writing: true,
    }
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
    if self.writing {
      self.hooks.push(f(self.component_id));
    }

    let value = self.hooks.get(self.index).ok_or(Error::InvalidIndex(self.index))?.downcast_ref()?;

    self.index += 1;

    Ok(value)
  }

  /// Ends a cursor and returns its hooks.
  pub fn hooks(self) -> Vec<Hook> {
    self.hooks
  }
}
