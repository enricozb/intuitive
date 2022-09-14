//! Structures for working with styled text.

use std::ops::Deref;

use tui::text::{Span as TuiSpan, Spans as TuiSpans};

use crate::style::Style;

/// Text with a specific style.
#[derive(Default, Clone)]
pub struct Span {
  pub text: String,
  pub style: Style,
}

impl Span {
  pub fn new<Str: Into<String>, Sty: Into<Style>>(text: Str, style: Sty) -> Self {
    Self {
      text: text.into(),
      style: style.into(),
    }
  }
}

impl<S: Into<String>> From<S> for Span {
  fn from(s: S) -> Self {
    Self {
      text: s.into(),
      style: Style::default(),
    }
  }
}

impl<'a> From<&'a Span> for TuiSpan<'a> {
  fn from(span: &'a Span) -> Self {
    Self {
      content: (&span.text).into(),
      style: span.style.into(),
    }
  }
}

impl From<Span> for TuiSpan<'_> {
  fn from(span: Span) -> Self {
    Self {
      content: span.text.into(),
      style: span.style.into(),
    }
  }
}

/// Text with a variety of styles.
///
/// `Spans` can be thought of as `Vec<Span>`, as they implement [`IntoIterator`],
/// and [`Deref`] into a [`Vec`]. Usually, components that accept text (such as
/// [`Section`] accepting a title), will accept `Spans`, as they represent
/// text with potentially multiple styles. `Spans` also implement
/// `From<S: Into<Span>>`, making it easy to pass values of many types to components
/// accepting `Spans`, such as [`String`], [`Span`], and [`&str`].
///
/// [`Deref`]: https://doc.rust-lang.org/std/ops/trait.Deref.html
/// [`IntoIterator`]: https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
/// [`Section`]: ../components/struct.Section.html
/// [`Span`]: struct.Span.html
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
#[derive(Default, Clone)]
pub struct Spans(Vec<Span>);

impl Spans {
  pub fn new(spans: Vec<Span>) -> Self {
    Self(spans)
  }
}

impl<S: Into<Span>> From<S> for Spans {
  fn from(s: S) -> Self {
    Spans(vec![s.into()])
  }
}

impl<'a> From<&'a Spans> for TuiSpans<'a> {
  fn from(spans: &'a Spans) -> Self {
    TuiSpans(spans.0.iter().map(TuiSpan::from).collect())
  }
}

impl From<Spans> for TuiSpans<'_> {
  fn from(spans: Spans) -> Self {
    TuiSpans(spans.0.into_iter().map(TuiSpan::from).collect())
  }
}

impl IntoIterator for Spans {
  type Item = Span;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl Deref for Spans {
  type Target = Vec<Span>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Clone)]
pub struct Lines(pub Vec<Spans>);

impl<S: Into<Spans>> From<S> for Lines {
  fn from(spans: S) -> Self {
    let mut expanded = Vec::new();

    for span in spans.into() {
      let lines: Vec<&str> = span.text.split('\n').collect();
      expanded.push(Some(Span::new(lines[0], span.style)));

      for line in &lines[1..] {
        expanded.push(None);
        expanded.push(Some(Span::new(*line, span.style)));
      }
    }

    let split = expanded
      .split(|span| span.is_none())
      .map(|spans| Spans::new(spans.iter().flatten().cloned().collect::<Vec<Span>>()))
      .collect();

    Lines(split)
  }
}
