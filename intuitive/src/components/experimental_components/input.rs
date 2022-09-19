//! A module containing the `Input` component.

use std::cmp;

use tui::{text::Spans as TuiSpans, widgets::Paragraph};

use crate::{
  component,
  components::{stack::Flex::*, Empty, Section, VStack},
  element::{Any as AnyElement, Element},
  event::{KeyHandler, MouseHandler},
  on_key, render,
  state::use_state,
  style::Style,
  terminal::{Frame, Rect},
  text::Spans,
};

/// A single-line input component with cursor controls.
///
/// This is a smart input box with the following features:
///   - rendering a cursor
///   - scrolling on overflow
///   - supports navigating with arrow keys
///   - supports navigating with `ctrl+a` and `ctrl+e`
///   - has a fixed single-line height of 3 rows
///
/// ## Vertical Alignment
///
/// Since this component always renders as three blocks high,
/// when more space is available it vertically centers itself.
/// In order to align this element to the top, you will need
/// to use [`VStack`] along with [`Flex::Block`] and properly
/// route the key events:
///
/// ```rust
/// # use intuitive::{
/// #   component,
/// #   components::{stack::Flex::*, Embed, Empty, experimental::input::Input, VStack},
/// #   element,
/// #   error::Result,
/// #   on_key, render,
/// #   terminal::Terminal,
/// # };
/// #
/// #[component(Root)]
/// fn render() {
///   let input: element::Any = render! {
///     Input(title: "Input Box")
///   };
///
///   let on_key = on_key! { [input]
///     KeyEvent { code: Esc, .. } => event::quit(),
///     event => input.on_key(event),
///   };
///
///   render! {
///     VStack(flex: [Block(3), Grow(1)], on_key) {
///       Embed(content: input)
///       Empty()
///     }
///   }
/// }
/// ```
///
/// [`VStack`]: ../../struct.VStack.html
/// [`Flex::Block`]: ../../stack/enum.Flex.html#variant.Block
#[component(Input)]
pub fn render(title: Spans, border: Style, on_key: KeyHandler, on_mouse: MouseHandler) {
  let cursor = use_state(|| 0usize);
  let text = use_state(|| String::new());

  let on_key = on_key.then(on_key! { [cursor, text]
    KeyEvent { code: Char('a'), modifiers: KeyModifiers::CONTROL, .. } => cursor.set(0),
    KeyEvent { code: Char('e'), modifiers: KeyModifiers::CONTROL, .. } => cursor.set(text.get().len()),

    KeyEvent { code: Left, .. } => {
      cursor.update(|cursor| cursor.saturating_sub(1));
    },

    KeyEvent { code: Right, .. } => {
      cursor.update(|cursor| cmp::min(cursor + 1, text.get().len()));
    },

    KeyEvent { code: Char(c), .. } => {
      text.mutate(|text| text.insert(cursor.get(), c));
      cursor.update(|cursor| cursor + 1);
    },

    KeyEvent { code: Backspace, .. } => {
      if cursor.get() > 0 && text.get().len() > 0 {
        text.mutate(|text| text.remove(cursor.get() - 1));
        cursor.update(|cursor| cursor - 1);
      }
    },
  });

  // TODO(enricozb): consider adding (Vertical)Alignment as a property, that will
  // align the input box to the top/middle/bottom.
  render! {
    VStack(flex: [Grow(1), Block(3), Grow(1)], on_key) {
      Empty()
      Section(title, border, on_mouse) {
        Inner(cursor: cursor.get(), text: text.get())
      }
      Empty()
    }
  }
}

#[component(Inner)]
fn render(cursor: usize, text: String) {
  AnyElement::new(Frozen {
    cursor: *cursor as u16,
    text: text.clone(),
  })
}

struct Frozen {
  cursor: u16,
  text: String,
}

impl Element for Frozen {
  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let (text, cursor) = if self.cursor < rect.width {
      (self.text.clone().into(), rect.x + self.cursor)
    } else {
      let offset = (self.cursor - rect.width) as usize + 1;
      (self.text[offset..].into(), rect.right() - 1)
    };

    let widget = Paragraph::new::<TuiSpans>(text);
    frame.render_widget(widget, rect);
    frame.set_cursor(cursor, rect.y);
  }
}
