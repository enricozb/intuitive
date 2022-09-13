use tui::{
  text::Spans as TuiSpans,
  widgets::{Block, Borders},
};

use crate::{
  components::{children::Children, Component},
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
#[derive(Clone, Default)]
pub struct Section {
  pub title: Spans,
  pub border: Style,

  pub children: Children<1>,

  pub on_key: KeyHandler,
  pub on_mouse: MouseHandler,
}

impl Component for Section {
  fn render(&self) -> AnyElement {
    AnyElement::new(Frozen {
      title: self.title.clone(),
      border: self.border,

      content: self.children[0].render(),
      on_key: self.on_key.clone(),
      on_mouse: self.on_mouse.clone(),
    })
  }
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
      )
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
