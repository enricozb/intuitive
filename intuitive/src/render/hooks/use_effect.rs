use crate::render::hooks::{manager, Hook};

/// A hook to execute a function once within a component.
///
/// Executes the provided `func` once (when the component is first rendered), and never again.
///
/// # Future Work
///
/// This function should accept a `deps: D` parameter that would behave more like React's [`useEffect`],
/// where it executes any time a re-render occurs and `deps` has changed since the last render.
///
/// [`useEffect`]: https://reactjs.org/docs/hooks-effect.html
pub fn use_effect<F, T>(func: F)
where
  F: FnOnce() -> T,
{
  manager::use_hook::<()>(|_| {
    drop(func());

    Hook::from_value(())
  })
  .expect("use_effect: use_hook");
}
