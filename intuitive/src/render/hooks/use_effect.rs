#[allow(unused)]
use crate::render::hooks::UseEffectWithDeps;
use crate::render::{hooks::Hook, providers::Hooks};

/// Optional function to run as a cleanup for [`UseEffect`] or [`UseEffectWithDeps`].
#[derive(Default)]
pub struct Cleanup(Option<Box<dyn FnOnce()>>);

impl Cleanup {
  /// Consumes and calls the cleanup function.
  pub(crate) fn call(self) {
    if let Some(cleanup) = self.0 {
      cleanup();
    }
  }
}

impl From<()> for Cleanup {
  fn from((): ()) -> Self {
    Self(None)
  }
}

impl<F> From<F> for Cleanup
where
  F: 'static + FnOnce(),
{
  fn from(cleanup: F) -> Self {
    Self(Some(Box::new(cleanup)))
  }
}

/// A hook to execute a function once within a component.
pub trait UseEffect {
  /// Executes the provided `func` once (when the component is first rendered), and never again.
  /// The function can optionally return a [`Cleanup`], which will be run when its parent component
  /// is unmounted.
  ///
  /// See also [`UseEffectWithDeps`](UseEffectWithDeps).
  ///
  /// This is inspired by React's [`useEffect`].
  ///
  /// [`useEffect`]: https://reactjs.org/docs/hooks-effect.html
  fn use_effect<F, T>(&mut self, func: F)
  where
    F: FnOnce() -> T,
    T: Into<Cleanup>;
}

impl UseEffect for Hooks {
  fn use_effect<F, T>(&mut self, func: F)
  where
    F: FnOnce() -> T,
    T: Into<Cleanup>,
  {
    self
      .use_hook::<()>(|_| match func().into() {
        Cleanup(Some(cleanup)) => Hook::new((), cleanup),
        Cleanup(None) => Hook::from_value(()),
      })
      .expect("use_effect: use_hook");
  }
}
