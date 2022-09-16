use tui::{
  text::Spans as TuiSpans,
  widgets::{Block, Borders},
};

use crate::{
  component,
  components::children::Children,
  element::{Any as AnyElement, Element},
  event::{KeyEvent, KeyHandler, MouseEvent, MouseHandler},
  style::Style,
  terminal::{Frame, Rect},
  text::Spans,
};

/// A component with a border and a title.
///
/// `Section` is used to wrap a component with a border and a title.
/// For example,
/// ```rust
/// # use intuitive::{component, components::{Section, Text}, render};
/// #
/// #[component(Root)]
/// fn render() {
///   render! {
///     Section(title: "Input Box") {
///       Text(text: "Hi there!")
///     }
///   }
/// }
/// ```
/// Will render the following:
///
/// ![section](https://raw.githubusercontent.com/enricozb/intuitive/main/assets/section.png)
///
/// `Section` also accepts a border [`Style`]. This style will merge with any style applied to
/// the title.
///
/// [`Style`]: ../style/struct.Style.html
#[component(Section)]
pub fn render(title: Spans, border: Style, children: Children<1>, on_key: KeyHandler, on_mouse: MouseHandler) {
  AnyElement::new(Frozen {
    title: title.clone(),
    border: *border,

    content: children[0].render(),
    on_key: on_key.clone(),
    on_mouse: on_mouse.clone(),
  })
}

struct Frozen {
  title: Spans,
  border: Style,

  content: AnyElement,
  on_key: KeyHandler,
  on_mouse: MouseHandler,
}

impl Element for Frozen {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle_or(event, |event| self.content.on_key(event));
  }

  fn on_mouse(&self, rect: Rect, event: MouseEvent) {
    self.on_mouse.handle_or(event, |event| {
      self.content.on_mouse(
        Rect {
          x: rect.x + 1,
          y: rect.y - 1,
          width: rect.width - 1,
          height: rect.height - 1,
        },
        event,
      );
    });
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let block = Block::default()
      .title::<TuiSpans>((&self.title).into())
      .borders(Borders::ALL)
      .border_style(self.border.into());

    self.content.draw(block.inner(rect), frame);
    frame.render_widget(block, rect);
  }
}
