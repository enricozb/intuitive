use std::{cell::Cell, rc::Rc};

use parking_lot::Mutex;

#[allow(unused)]
use crate::render::hooks::UseEffect;
use crate::render::{
  hooks::{Cleanup, Hook},
  providers::Hooks,
};

/// The inner hook value to [`UseEffectWithDeps::use_effect_with_deps`].
struct Effect<D> {
  deps: Rc<Mutex<D>>,
  cleanup: Rc<Cell<Cleanup>>,
}

impl<D: Eq> Effect<D> {
  /// Swaps the inner [`Self::deps`] and [`Self::cleanup`] if `new_deps` differs from the current [`Self::deps`].
  fn swap_if_neq<F, T>(&self, func: F, new_deps: D)
  where
    F: FnOnce() -> T,
    T: Into<Cleanup>,
  {
    let mut old_deps = self.deps.lock();
    if *old_deps != new_deps {
      *old_deps = new_deps;

      self.cleanup.replace(func().into());
    }
  }
}

impl<D> Clone for Effect<D> {
  fn clone(&self) -> Self {
    Self {
      deps: self.deps.clone(),
      cleanup: self.cleanup.clone(),
    }
  }
}

impl<F, T, D> From<(F, D)> for Effect<D>
where
  F: FnOnce() -> T,
  T: Into<Cleanup>,
  D: Eq,
{
  fn from((func, deps): (F, D)) -> Self {
    Self {
      deps: Rc::new(Mutex::new(deps)),
      cleanup: Rc::new(Cell::new(func().into())),
    }
  }
}

impl<D: 'static> From<Effect<D>> for Hook {
  fn from(effect: Effect<D>) -> Self {
    let cleanup = effect.cleanup.clone();

    Self::new(effect, move || cleanup.take().call())
  }
}

/// A hook to execute a function any time its dependencies changes within a component.
pub trait UseEffectWithDeps {
  /// Executes the provided `func` once (when the component is first rendered), and again any time `deps` differs
  /// from its previous value. The function can optionally return a [`Cleanup`], which will be run anytime this
  /// function is re-run, or when its parent component is unmounted.
  ///
  /// See also [`UseEffect`](UseEffect).
  ///
  /// This is inspired by React's [`useEffect`].
  ///
  /// [`useEffect`]: https://reactjs.org/docs/hooks-effect.html
  fn use_effect_with_deps<F, T, D>(&mut self, deps: D, func: F)
  where
    D: 'static + Eq,
    F: FnOnce() -> T,
    T: Into<Cleanup>;
}

impl UseEffectWithDeps for Hooks {
  fn use_effect_with_deps<F, T, D>(&mut self, deps: D, func: F)
  where
    D: 'static + Eq,
    F: FnOnce() -> T,
    T: Into<Cleanup>,
  {
    if self.has_hook().expect("use_effect_with_deps: has_hook") {
      // prevent from moving func and deps
      let effect: Effect<D> = self.use_hook(|_| Hook::default()).expect("use_effect_with_deps: use_hook");

      effect.swap_if_neq(func, deps);
    } else {
      self
        .use_hook::<Effect<D>>(move |_| Effect::from((func, deps)).into())
        .expect("use_effect_with_deps: use_hook");
    }
  }
}
