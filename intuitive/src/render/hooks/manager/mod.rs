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

/// Calls [`Manager::use_hook`] for the global [`Manager`].
#[allow(rustdoc::private_intra_doc_links)]
pub fn use_hook<I, T>(initializer: I) -> Result<T>
where
  T: 'static + Send + Sync + Clone,
  I: Initializer<T>,
{
  MANAGER.use_hook(initializer)
}

/// Manages [`use_hook`] calls across renders.
///
/// [`Manager`]s have interior mutability, so they can be [`Sync`].
pub struct Manager {
  /// A stack of [`Cursor`]s that are pushed/popped before/after rendering.
  cursors: Mutex<Vec<Cursor>>,
  /// Maps instances of components within [`render!`] calls to memoized hook initializer return values.
  memos: Mutex<HashMap<ComponentID, Memos>>,
}

impl Manager {
  /// Creates a new [`Manager`].
  fn new() -> Self {
    Self {
      cursors: Mutex::new(Vec::new()),
      memos: Mutex::new(HashMap::new()),
    }
  }

  /// Calls `f` with a [`Cursor`] for the given [`ComponentID`] at the top of the [`Self::cursors`] stack.
  fn with<F, T>(&self, component_id: ComponentID, f: F) -> T
  where
    F: FnOnce() -> T,
  {
    let mode = match self.memos.lock().entry(component_id) {
      Entry::Occupied(_) => Mode::Reading,
      Entry::Vacant(e) => {
        e.insert(Memos::new());
        Mode::Writing
      }
    };

    self.cursors.lock().push(Cursor::new(component_id, mode));
    let ret = f();
    self.cursors.lock().pop();

    ret
  }

  /// Returns the return value of the provided [`Initializer`], memoizing it if it was called previously.
  fn use_hook<I, T>(&self, initializer: I) -> Result<T>
  where
    T: 'static + Send + Sync + Clone,
    I: Initializer<T>,
  {
    let mut cursors = self.cursors.lock();
    let cursor = cursors.last_mut().ok_or(Error::NoCursor)?;

    let val = match cursor.mode {
      Mode::Reading => self
        .memos
        .lock()
        .get(&cursor.component_id)
        .ok_or(Error::NoMemo(cursor.component_id))?
        .get::<T>(cursor.idx)?,

      Mode::Writing => {
        let val = initializer(cursor.component_id);

        self
          .memos
          .lock()
          .get_mut(&cursor.component_id)
          .ok_or(Error::NoMemo(cursor.component_id))?
          .push(val.clone());

        val
      }
    };

    cursor.increment();

    Ok(val)
  }
}
