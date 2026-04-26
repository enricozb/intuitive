//! Signals: cheap, cloneable handles to per-element reactive state.
//!
//! `Signal<T>` wraps `Arc<RwLock<T>>` for direct shared access. The
//! runtime retains a type-erased `Arc` to the same storage in
//! `Runtime.signals` so the storage outlives the component body's local
//! handles and can be dropped at unmount.
//!
//! Reads:
//! - `get(&mut Context)` reads under the lock and subscribes the
//!   currently-rendering element. Returns `T` by clone.
//! - `map(&mut Context, |&T| -> R)` subscribes and runs a closure with a
//!   reference to the value under the lock. Avoids `T: Clone` when the caller
//!   only needs a projection (e.g. a field, `.len()`).
//! - `peek()` reads under the lock without subscribing. Callable from anywhere
//!   (event handlers, spawned tasks, other threads).
//!
//! Writes:
//! - `set(value)` and `update(|&mut T| ...)` mutate under the lock and post an
//!   `Event::SignalChanged(id)` notification onto the event channel. Storage
//!   changes synchronously; dirty propagation and re-rendering happen on the
//!   next event-loop tick.

use std::{
  any::Any,
  panic::Location,
  sync::{Arc, RwLock},
};

use tokio::sync::mpsc::UnboundedSender;

use crate::runtime::{ElementKey, Event};

/// Identifies a hook slot within an element. Wraps the
/// `&'static Location` returned by `Location::caller()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SlotId(pub &'static Location<'static>);

/// Identifies a particular signal's storage in `Runtime.signals`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SignalId {
  pub element: ElementKey,
  pub slot: SlotId,
}

/// A cheap, cloneable, sendable handle to a signal. All clones reference
/// the same shared storage cell.
pub struct Signal<T> {
  id: SignalId,
  cell: Arc<RwLock<T>>,
  tx: UnboundedSender<Event>,
}

impl<T> Clone for Signal<T> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      cell: Arc::clone(&self.cell),
      tx: self.tx.clone(),
    }
  }
}

impl<T> Signal<T> {
  pub(crate) fn from_parts(id: SignalId, cell: Arc<RwLock<T>>, tx: UnboundedSender<Event>) -> Self {
    Self { id, cell, tx }
  }

  #[must_use]
  pub fn id(&self) -> &SignalId {
    &self.id
  }
}

impl<T: Send + Sync + 'static> Signal<T> {
  /// Read and subscribe. Panics on poisoned lock.
  pub fn get(&self, ctx: &mut crate::runtime::Context<'_>) -> T
  where
    T: Clone,
  {
    ctx.subscribe(self.id.clone());
    self.peek()
  }

  /// Subscribe and run `f` with a reference to the value, under the
  /// read lock. Returns whatever `f` returns.
  ///
  /// Useful when `T` is not `Clone`, or when only a projection of `T`
  /// is needed (e.g. a field, `.len()`, a derived `bool`):
  ///
  /// ```ignore
  /// let len = items.map(ctx, |v: &Vec<String>| v.len());
  /// let name = user.map(ctx, |u: &User| u.name.clone());
  /// ```
  ///
  /// The lock is held only for the duration of `f`. Do not call
  /// `Signal::set` / `Signal::update` on *this* signal from inside
  /// `f` — it would deadlock waiting for the write lock.
  ///
  /// # Panics
  ///
  /// Panics if the underlying lock has been poisoned.
  pub fn map<R>(&self, ctx: &mut crate::runtime::Context<'_>, f: impl FnOnce(&T) -> R) -> R {
    ctx.subscribe(self.id.clone());
    let guard = self.cell.read().expect("signal lock poisoned");
    f(&*guard)
  }

  /// Read without subscribing. Callable from any context.
  ///
  /// # Panics
  ///
  /// Panics if the underlying lock has been poisoned by a panic in
  /// another thread while holding it.
  #[must_use]
  pub fn peek(&self) -> T
  where
    T: Clone,
  {
    self.cell.read().expect("signal lock poisoned").clone()
  }

  /// Replace the value. Mutates synchronously, then posts a notification.
  ///
  /// # Panics
  ///
  /// Panics if the underlying lock has been poisoned.
  pub fn set(&self, value: T) {
    {
      let mut guard = self.cell.write().expect("signal lock poisoned");
      *guard = value;
    }
    let _ = self.tx.send(Event::Signal(self.id.clone()));
  }

  /// Read-modify-write under the lock. After `f` returns, posts a
  /// notification.
  ///
  /// # Panics
  ///
  /// Panics if the underlying lock has been poisoned.
  pub fn update<F>(&self, f: F)
  where
    F: FnOnce(&mut T),
  {
    {
      let mut guard = self.cell.write().expect("signal lock poisoned");
      f(&mut *guard);
    }
    let _ = self.tx.send(Event::Signal(self.id.clone()));
  }
}

/// Type-erased shared storage entry retained by `Runtime.signals`.
/// Concretely an `Arc<RwLock<T>>` for the signal's `T`, type-erased to
/// `Arc<dyn Any + Send + Sync>` for storage in a heterogeneous map.
pub(crate) type ErasedSignal = Arc<dyn Any + Send + Sync>;
