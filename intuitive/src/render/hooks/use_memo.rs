use std::rc::Rc;

use parking_lot::Mutex;

#[allow(unused)]
use crate::render::hooks::UseEffect;
use crate::render::{hooks::Hook, providers::Hooks};

/// A memoization entry along with its dependencies.
struct Memo<D, T> {
  deps: Rc<Mutex<D>>,
  memo: Rc<Mutex<T>>,
}

impl<D, T: Clone> Memo<D, T> {
  /// Returns a clone of the memoized value.
  fn value(&self) -> T {
    self.memo.lock().clone()
  }
}

impl<D: Eq, T> Memo<D, T> {
  /// Swaps the inner [`Self::deps`] and [`Self::memo`] if `new_deps` differs from the current [`Self::deps`].
  fn swap_if_neq<F>(&self, func: F, new_deps: D)
  where
    F: FnOnce() -> T,
  {
    let mut old_deps = self.deps.lock();
    if *old_deps != new_deps {
      *old_deps = new_deps;

      *self.memo.lock() = func();
    }
  }
}

impl<D, T> Clone for Memo<D, T> {
  fn clone(&self) -> Self {
    Self {
      deps: self.deps.clone(),
      memo: self.memo.clone(),
    }
  }
}

impl<F, T, D> From<(F, D)> for Memo<D, T>
where
  F: FnOnce() -> T,
  D: Eq,
{
  fn from((func, deps): (F, D)) -> Self {
    Self {
      deps: Rc::new(Mutex::new(deps)),
      memo: Rc::new(Mutex::new(func())),
    }
  }
}

impl<D: 'static, T: 'static> From<Memo<D, T>> for Hook {
  fn from(memo: Memo<D, T>) -> Self {
    Self::from_value(memo)
  }
}

/// A hook to avoid recomputing an expensive function.
pub trait UseMemo {
  /// Returns `func()`, which is initially executed only once (when the component is first rendered), and again any
  /// time `deps` differs from its previous value.
  ///
  /// This is inspired by React's [`useMemo`].
  ///
  /// [`useMemo`]: https://reactjs.org/docs/hooks-reference.html#usememo
  fn use_memo<D, F, T>(&mut self, deps: D, func: F) -> T
  where
    D: 'static + Eq,
    F: FnOnce() -> T,
    T: 'static + Clone;
}

impl UseMemo for Hooks {
  fn use_memo<D, F, T>(&mut self, deps: D, func: F) -> T
  where
    D: 'static + Eq,
    F: FnOnce() -> T,
    T: 'static + Clone,
  {
    if self.has_hook().expect("use_memo: has_hook") {
      // prevent from moving func and deps
      let memo: Memo<D, T> = self.use_hook(|_| Hook::default()).expect("use_memo: use_hook");

      memo.swap_if_neq(func, deps);

      memo.value()
    } else {
      let memo = self
        .use_hook::<Memo<D, T>>(move |_| Memo::from((func, deps)).into())
        .expect("use_memo: use_hook");

      memo.value()
    }
  }
}
