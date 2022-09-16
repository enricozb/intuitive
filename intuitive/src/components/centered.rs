use crate::{
  component,
  components::{children::Children, Embed, Empty, HStack, VStack},
  event::KeyHandler,
  on_key, render,
};

/// A component for centering its contents.
///
/// For example,
/// ```rust
/// # use intuitive::{component, components::{Centered, Section}, render};
/// #
/// #[component(Root)]
/// fn render() {
///   render! {
///     Centered() {
///       Section(title: "I'm centered")
///     }
///   }
/// }
/// ```
#[component(Centered)]
pub fn render(children: Children<1>, on_key: KeyHandler) {
  let child = children[0].render();

  let on_key = on_key! { [child, on_key]
    event => on_key.handle_or(event, |event| child.on_key(event))
  };

  render! {
    VStack(on_key) {
      Empty()
      HStack() {
        Empty()
        Embed(content: child)
        Empty()
      }
      Empty()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{components::Text, element::Any as AnyElement, state::State};

  #[test]
  fn centered_forwards_keys() {
    pub use crate::event::{KeyCode::*, KeyEvent, KeyModifiers};

    let called = State::new(false);
    let on_key_called = called.clone();

    let on_key = on_key! {
      KeyEvent { code: Esc, .. } => on_key_called.set(true),
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
