use tui::{
  buffer::Buffer,
  layout::{Constraint, Rect},
  style::{Modifier, Style},
  text::{Span, Spans},
  widgets::{Cell as TuiCell, Row, Table as TuiTable, Widget},
};

use super::alignment::Alignment;

type Rows<'a, const N: usize> = Vec<[Spans<'a>; N]>;

pub struct Table<'a, const N: usize> {
  pub rows: Rows<'a, N>,
  pub alignments: [Alignment; N],
}

impl<'a, const N: usize> Table<'a, N> {
  pub fn new(rows: Rows<'a, N>, alignments: [Alignment; N]) -> Self {
    Self { rows, alignments }
  }

  fn lengths(&self) -> Vec<[usize; N]> {
    self
      .rows
      .iter()
      .map(|row| {
        let mut lengths = [0; N];
        for (i, cell) in row.iter().enumerate() {
          lengths[i] = Spans::width(cell);
        }

        lengths
      })
      .collect()
  }

  #[allow(clippy::cast_possible_truncation)]
  fn constraints(&self) -> [Constraint; N] {
    let lengths: Vec<[usize; N]> = self.lengths();
    let mut constraints = [0; N];
    for i in 0..N {
      constraints[i] = lengths.iter().map(|l| l[i]).max().unwrap_or_default() as u16;
    }

    constraints.map(Constraint::Length)
  }

  fn aligned_rows(rows: Rows<'a, N>, alignments: &[Alignment; N], constraints: &[Constraint; N]) -> Vec<Row<'a>> {
    rows
      .into_iter()
      .map(|mut row| {
        for i in 0..N {
          let (spans, alignment, constraint) = (&mut row[i], &alignments[i], constraints[i]);
          if *alignment == Alignment::Right {
            if let Constraint::Length(width) = constraint {
              spans.0.insert(0, Span::raw(" ".repeat((width as usize) - spans.width())));
            }
          }
        }

        Row::new(row.into_iter().map(TuiCell::from).collect::<Vec<_>>())
      })
      .collect()
  }

  fn widget(self, constraints: &'a [Constraint; N]) -> TuiTable<'a> {
    TuiTable::new(Self::aligned_rows(self.rows, &self.alignments, constraints))
      .highlight_style(Style::default().add_modifier(Modifier::BOLD))
      .widths(constraints)
  }
}

impl<'a, const N: usize> Widget for Table<'a, N> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let constraints = self.constraints();

    Widget::render(self.widget(&constraints), area, buf);
  }
}
