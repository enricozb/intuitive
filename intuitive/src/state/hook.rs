use parking_lot::Mutex;

use super::manager::Manager;
pub use super::State;
use crate::error::Error;

static MANAGER: Mutex<Manager> = Mutex::new(Manager::new());

pub fn render_done() {
  MANAGER.lock().reset().map_err(|err| Error::UseState(err.to_string())).unwrap();
}

/// A hook for managing state within a [`Component`]
///
/// Similarly to [React Hooks], `use_state` lets you manager state without an explicit
/// struct storing state. `use_state` returns a [`State`], which can be used to retrieve
/// or update the value. For example,
/// ```rust
/// # use intuitive::{component, components::{Section, Text}, state::use_state, on_key, render};
/// #
/// #[component(Input)]
/// fn render(title: String) {
///   let text = use_state(|| String::new());
///
///   let on_key = on_key! { [text]
///     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
///     KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
///   };
///
///   render! {
///     Section(title) {
///       Text(text: text.get(), on_key)
///     }
///   }
/// }
/// ```
///
/// # Gotchas
/// Any calls to `use_state` must always be called in the same order and in every render.
/// This means that there can not be any conditional logic around the calling of `use_state`.
/// If Intuitive detects such a violation, it will panic with an appropriate message.
///
/// [`Component`]: ../components/trait.Component.html
/// [`State`]: struct.State.html
/// [React Hooks]: https://reactjs.org/docs/hooks-intro.html
pub fn use_state<T, F>(initializer: F) -> State<T>
where
  T: 'static + Send,
  F: FnOnce() -> T,
{
  MANAGER
    .lock()
    .next(initializer)
    .map_err(|err| Error::UseState(err.to_string()))
    .unwrap()
}

#[cfg(test)]
mod tests {
  use serial_test::serial;

  use super::*;

  fn setup() {
    *MANAGER.lock() = Manager::new();

    let _ = use_state(|| 1);
    let _ = use_state(|| 2);

    render_done();
  }

  #[test]
  #[serial]
  fn use_state_no_panic() {
    setup();

    let _ = use_state(|| 1);
    let _ = use_state(|| 2);

    render_done();
  }

  #[test]
  #[serial]
  fn use_state_get() {
    setup();

    let state_1 = use_state(|| 1);
    let state_2 = use_state(|| 2);

    assert_eq!(state_1.get(), 1);
    assert_eq!(state_2.get(), 2);

    render_done();
  }

  #[test]
  #[serial]
  fn use_state_set_get() {
    setup();

    let state_1 = use_state(|| 1);
    let state_2 = use_state(|| 2);

    state_1.set(3);
    state_2.set(4);

    assert_eq!(state_1.get(), 3);
    assert_eq!(state_2.get(), 4);

    render_done();
  }

  #[test]
  #[serial]
  #[should_panic]
  fn use_state_wrong_type() {
    setup();

    let _ = use_state(|| ());

    render_done();
  }

  #[test]
  #[serial]
  #[should_panic]
  fn use_state_too_few() {
    setup();

    let _ = use_state(|| 1);

    render_done();
  }

  #[test]
  #[serial]
  #[should_panic]
  fn use_state_too_many() {
    setup();

    let _ = use_state(|| 1);
    let _ = use_state(|| 2);
    let _ = use_state(|| 3);

    render_done();
  }
}
