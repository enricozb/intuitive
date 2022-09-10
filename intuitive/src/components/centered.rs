use crate::{
  component,
  components::{children::Children, Embed, Empty, HStack, VStack},
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler},
  render,
  terminal::{Frame, Rect},
};

/// A component for centering its contents.
///
/// For example,
/// ```rust
/// render! {
///   Centered() {
///     Section(title: "I'm centered")
///   }
/// }
/// ```
#[component(crate::Centered)]
pub fn render(children: Children<1>, on_key: KeyHandler) {
  let child = children[0].render();
  let child_clone = child.clone();
  let on_key_clone = on_key.clone();

  let on_key = move |event| {
    on_key_clone.handle_or(event, |event| child_clone.on_key(event));
  };

  render! {
    VStack(on_key) {
      Empty()
      HStack() {
        Empty()
        Embed(content: child.clone())
        Empty()
      }
      Empty()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{components::Text, state::State};

  #[test]
  fn centered_forwards_keys() {
    pub use crate::event::{KeyCode::*, KeyEvent, KeyModifiers};

    let called = State::new(false);
    let on_key_called = called.clone();

    let on_key = move |event| match event {
      KeyEvent { code: Esc, .. } => on_key_called.set(true),
      _ => (),
    };

    let centered: AnyElement = render! {
      Centered() {
        Text(on_key)
      }
    };

    centered.on_key(KeyEvent::new(Esc, KeyModifiers::NONE));

    assert_eq!(called.get(), true);
  }
}
