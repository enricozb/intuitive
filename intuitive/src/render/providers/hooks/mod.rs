mod cursor;
mod error;

use std::{
  any::{self, Any},
  collections::{HashMap, HashSet},
};

use self::{
  cursor::Cursor,
  error::{Error, Result},
};
use crate::{render::ComponentID, utils::provider::Provider};

/// A dynamically-typed hook return value, along with a deinitialization function for unmounting.
pub struct Hook {
  /// The inner value of the hook.
  value: Box<dyn Any>,

  /// Any deinitialization needed for whne this hook's parent component is unmounted.
  deinit: Option<Box<dyn FnOnce()>>,
}

impl Hook {
  /// Creates a new [`Hook`].
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

/// Hooks used within renders.
pub struct Hooks {
  cursors: Vec<Cursor>,
  hooks: HashMap<ComponentID, Vec<Hook>>,
}

impl Hooks {
  pub fn new() -> Self {
    Self {
      cursors: Vec::new(),
      hooks: HashMap::new(),
    }
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

  fn deinit(&mut self, unmounted_component_ids: HashSet<ComponentID>) {
    for component_id in unmounted_component_ids {
      for hook in self.hooks.remove(&component_id).unwrap_or_default() {
        hook.deinit();
      }
    }
  }
}

impl Provider for Hooks {
  type Entry = ComponentID;
  type Context = ();
  /// The component ids that were unmounted, to call [`Hook::deinit`] on.
  type Exit = HashSet<ComponentID>;
  type Output = Result<()>;

  fn enter(&mut self, component_id: Self::Entry) -> Self::Context {
    let cursor = match self.hooks.remove(&component_id) {
      Some(hooks) => Cursor::read(component_id, hooks),
      None => Cursor::write(component_id),
    };

    self.cursors.push(cursor);
  }

  fn exit(&mut self, (): Self::Context, unmounted_component_ids: Self::Exit) -> Self::Output {
    let cursor = self.cursors.pop().ok_or(Error::NoCursor)?;

    self.hooks.insert(cursor.component_id, cursor.hooks());

    self.deinit(unmounted_component_ids);

    Ok(())
  }
}
