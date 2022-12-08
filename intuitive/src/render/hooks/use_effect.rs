use crate::render::hooks::{manager::Manager as HookManager, Hook};

/// Optional function to run as a "cleanup" when a [`UseEffect::use_effect`] hook is unmounted.
pub struct Cleanup(Option<Box<dyn FnOnce() + Send + Sync>>);

impl From<()> for Cleanup {
  fn from((): ()) -> Self {
    Self(None)
  }
}

impl<F> From<F> for Cleanup
where
  F: 'static + FnOnce() + Send + Sync,
{
  fn from(cleanup: F) -> Self {
    Self(Some(Box::new(cleanup)))
  }
}

/// A hook to execute a function once within a component.
pub trait UseEffect {
  /// Executes the provided `func` once (when the component is first rendered), and never again. If the return
  /// value to the function provided to [`UseEffect::use_effect`] is a function, it will be run as a "cleanup" once this
  /// hook's parent component is unmounted. See [`Cleanup`].
  ///
  /// # Future Work
  ///
  /// This function should accept a `deps: D` parameter that would behave more like React's [`useEffect`],
  /// where it executes any time a re-render occurs and `deps` has changed since the last render.
  ///
  /// [`useEffect`]: https://reactjs.org/docs/hooks-effect.html
  fn use_effect<F, T>(&mut self, func: F)
  where
    T: Into<Cleanup>,
    F: FnOnce() -> T;
}

impl UseEffect for HookManager {
  fn use_effect<F, T>(&mut self, func: F)
  where
    T: Into<Cleanup>,
    F: FnOnce() -> T,
  {
    self
      .use_hook::<()>(|_| match func().into() {
        Cleanup(Some(cleanup)) => Hook::new((), cleanup),
        Cleanup(None) => Hook::from_value(()),
      })
      .expect("use_effect: use_hook");
  }
}
