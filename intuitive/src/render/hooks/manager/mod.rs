mod cursor;
use std::collections::{hash_map::Entry, HashMap};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use self::cursor::{Cursor, Mode};
use super::{
  error::{Error, Result},
  Initializer, Memos,
};
#[allow(unused)]
use crate::render;
use crate::render::ComponentID;

lazy_static! {
  /// The global hook [`Manager`].
  static ref MANAGER: Mutex<Manager> = Mutex::new(Manager::new());
}

/// Calls [`Manager::with`] for the global [`MANAGER`].
pub fn with<F, T>(id: ComponentID, f: F) -> T
where
  F: FnOnce() -> T,
{
  MANAGER.lock().with(id, f)
}

/// Manages [`use_hook`] calls across renders.
pub struct Manager {
  /// A stack of [`Cursor`]s that are pushed/popped before/after rendering.
  cursors: Vec<Cursor>,
  /// Maps instances of components within [`render!`] calls to memoized hook initializer return values.
  memos: HashMap<ComponentID, Memos>,
}

impl Manager {
  /// Creates a new [`Manager`].
  fn new() -> Self {
    Self {
      cursors: Vec::new(),
      memos: HashMap::new(),
    }
  }

  /// Calls `f` with a [`Cursor`] for the given [`ComponentID`] at the top of the [`Self::cursor`]s stack.
  fn with<F, T>(&mut self, id: ComponentID, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    let mode = match self.memos.entry(id) {
      Entry::Occupied(_) => Mode::Reading,
      Entry::Vacant(e) => {
        e.insert(Memos::new());
        Mode::Writing
      }
    };

    self.cursors.push(Cursor::new(id, mode));
    let ret = f();
    self.cursors.pop();

    ret
  }

  /// Returns the return value of the provided [`Initializer`], memoizing it if it was called previously.
  fn use_hook<I, T>(&mut self, initializer: I) -> Result<T>
  where
    T: 'static + Send + Sync + Clone,
    I: Initializer<T>,
  {
    let Cursor { id, idx, mode } = self.cursors.last_mut().ok_or(Error::NoCursor)?;

    match mode {
      Mode::Reading => self.memos.get(id).ok_or(Error::NoMemo(*id))?.get::<T>(*idx),
      Mode::Writing => {
        let val = initializer(id);
        self.memos.get_mut(id).ok_or(Error::NoMemo(*id))?.push(val.clone());

        Ok(val)
      }
    }
  }
}
