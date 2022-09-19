//! Structures for working with styled text.

use std::ops::Deref;

use tui::text::{Span as TuiSpan, Spans as TuiSpans};

use crate::style::Style;

/// Text with a specific style.
///
/// As `Span` can only hold a single style, components typically accept [`Spans`]
/// or [`Lines`] when accepting text.
///
/// `Span` also implements `From<S: Into<String>>`, making it easy to pass
/// in string-like values to components accepting [`Span`], [`Spans`], and [`Lines`].
///
/// [`Lines`]: struct.Lines.html
/// [`Spans`]: struct.Spans.html
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

  pub fn len(&self) -> usize {
    self.text.len()
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

/// A single line of text with a variety of styles.
///
/// Components that accept multiple lines of text should accept [`Lines`].
/// Components that accept single lines of text should accept `Spans`.
///
/// `Spans` implement `From<S: Into<Span>>`, making it easy to pass values of
/// many types such as [`String`], [`Span`] and [`&str`].
///
/// [`Lines`]: struct.Lines.html
/// [`Span`]: struct.Span.html
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
#[derive(Default, Clone)]
pub struct Spans(pub Vec<Span>);

impl Spans {
  pub fn new(spans: Vec<Span>) -> Self {
    Self(spans)
  }

  pub fn len(&self) -> usize {
    self.0.iter().map(|span| span.len()).sum()
  }
}

impl<S: Into<Span>> From<S> for Spans {
  fn from(s: S) -> Self {
    Spans(vec![s.into()])
  }
}

impl<'a> From<&'a Spans> for Spans {
  fn from(spans: &'a Spans) -> Self {
    spans.clone()
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

impl Deref for Spans {
  type Target = Vec<Span>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Clone, Default)]
/// Multiple lines of text with a variety of styles.
///
/// Each [`Spans`] element in the inner [`Vec`] is considered a line. Components that
/// accept multiple lines of text should accept `Lines`. Components that accept single
/// lines of text should accept [`Spans`].
///
/// `Lines` implement `From<S: Into<Spans>>`, making it easy to pass values of
/// many types such as [`Spans`], [`Span`], [`String`] and [`&str`].
///
/// **Note**: The implementation of `From<S: Into<Spans>>` for `Lines` automatically
/// splits on newlines (`\n`). If you do not want this behavior, then construct
/// `Lines` directly.
///
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`Spans`]: struct.Spans.html
/// [`Span`]: struct.Span.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
pub struct Lines(pub Vec<Spans>);

impl<S: Into<Spans>> From<S> for Lines {
  fn from(spans: S) -> Self {
    let mut expanded = Vec::new();

    for span in spans.into().0 {
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
