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

  let content: AnyElement = render! {
    VStack() {
      Empty()
      HStack() {
        Empty()
        Embed(content: child.clone())
        Empty()
      }
      Empty()
    }
  };

  AnyElement::new(Frozen {
    child,
    content,
    on_key: on_key.clone(),
  })
}

struct Frozen {
  child: AnyElement,
  content: AnyElement,
  on_key: KeyHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle_or(event, |event| self.child.on_key(event));
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.content.draw(rect, frame);
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
