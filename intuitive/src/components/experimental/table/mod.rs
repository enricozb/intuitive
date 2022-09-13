//! A module containing the `Table` component and related structures.

mod alignment;
mod widget;

use tui::text::Spans as TuiSpans;

pub use self::alignment::{Alignment, Array as AlignmentArray};
use self::widget::Table as TableWidget;
use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
  event::{KeyCode, KeyEvent, KeyHandler},
  state::{use_state, State},
  terminal::{Frame, Rect},
  text::Spans,
};

/// A component to render tabular data.
#[derive(Default)]
pub struct Table<const N: usize> {
  pub alignments: AlignmentArray<N>,
  pub rows: Vec<[Spans; N]>,

  pub on_key: KeyHandler,
}

impl<const N: usize> Component for Table<N> {
  fn render(&self) -> AnyElement {
    let index = use_state(|| 0);

    AnyElement::new(Frozen {
      alignments: self.alignments,
      rows: self.rows.clone(),
      index,

      on_key: self.on_key.clone(),
    })
  }
}

struct Frozen<const N: usize> {
  alignments: AlignmentArray<N>,
  rows: Vec<[Spans; N]>,
  index: State<usize>,

  on_key: KeyHandler,
}

impl<const N: usize> Element for Frozen<N> {
  fn on_key(&self, event: KeyEvent) {
    self.on_key.handle_or(event, |event| {
      use KeyCode::*;

      match event {
        KeyEvent { code: Char('j'), .. } => self.index.update(|i| if i + 1 < self.rows.len() { i + 1 } else { *i }),
        KeyEvent { code: Char('k'), .. } => self.index.update(|i| i.saturating_sub(1)),

        _ => (),
      }
    });
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    let rows = self.rows.iter().cloned().map(|row| row.map(TuiSpans::from)).collect();
    let widget = TableWidget::new(rows, self.alignments.into());

    frame.render_widget(widget, rect);
  }
}
