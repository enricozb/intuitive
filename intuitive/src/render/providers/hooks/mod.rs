mod cursor;

use std::collections::{HashMap, HashSet};

use self::cursor::Cursor;
use crate::{
  render::{
    hooks::{
      error::{Error, Result},
      Hook,
    },
    ComponentID,
  },
  utils::provider::Provider,
};

/// Hooks used within renders.
#[derive(Default)]
pub struct Hooks {
  cursors: Vec<Cursor>,
  hooks: HashMap<ComponentID, Vec<Hook>>,
}

impl Hooks {
  /// Returns the inner value of the current [`Hook`], constructing it with `f` if necessary.
  ///
  /// The parameter `f` is not generic because `use_hook` is often used with a turbofish, and it
  /// would be difficult (impossible?) to specify the type of a closure.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if there is no [`Cursor`] at the top of the stack, or if
  /// [`Cursor::next`] returns an `Err`.
  #[allow(rustdoc::private_intra_doc_links)]
  pub fn use_hook<T>(&mut self, f: impl FnOnce(ComponentID) -> Hook) -> Result<T>
  where
    T: 'static + Clone,
  {
    self.cursors.last_mut().ok_or(Error::NoCursor)?.next(f)
  }

  /// Returns whether there is a hook to be read.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if there is no [`Cursor`] at the top of the stack.
  #[allow(rustdoc::private_intra_doc_links)]
  pub fn has_hook(&self) -> Result<bool> {
    Ok(self.cursors.last().ok_or(Error::NoCursor)?.has_hook())
  }

  /// Deinitializes all hooks for the given `component_ids`.
  fn deinit(&mut self, component_ids: &HashSet<ComponentID>) {
    for component_id in component_ids {
      for hook in self.hooks.remove(component_id).unwrap_or_default() {
        hook.deinit();
      }
    }
  }
}

impl Provider for Hooks {
  type Entry = ComponentID;
  type Context = ();
  /// Components that were unmounted, to call [`Hook::deinit`] on.
  type Exit = HashSet<ComponentID>;
  type Output = Result<()>;

  fn enter(&mut self, component_id: &Self::Entry) -> Self::Context {
    let cursor = match self.hooks.remove(component_id) {
      Some(hooks) => Cursor::read(*component_id, hooks),
      None => Cursor::write(*component_id),
    };

    self.cursors.push(cursor);
  }

  fn exit(&mut self, unmounted_component_ids: &Self::Exit) -> Self::Output {
    let cursor = self.cursors.pop().ok_or(Error::NoCursor)?;

    self.hooks.insert(cursor.component_id, cursor.hooks());

    self.deinit(unmounted_component_ids);

    Ok(())
  }
}
