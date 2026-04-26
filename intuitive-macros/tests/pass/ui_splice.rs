//! End-to-end pass test: splice syntax in `ui!{}` child blocks.
//!
//! Exercises all three splice forms:
//!
//! - `#name` \u2014 bare identifier substitution.
//! - `#(expr)` \u2014 arbitrary Rust expression of type `Element`.
//! - `#(expr)*` \u2014 iterator of `Element`.

use intuitive::{
  components::{HStack, Text, VStack},
  geometry::Flex,
  macros::ui,
  runtime::Element,
};

fn _build_ident_splice() -> Element {
  let inner: Element = ui! { Text(text: "spliced") };
  ui! {
    HStack(.flex: [Flex::Frac(1)]) {
      #inner
    }
  }
}

fn _build_expr_splice(items: Vec<String>) -> Element {
  let first = items.first().cloned().unwrap_or_default();
  ui! {
    HStack(.flex: [Flex::Frac(1), Flex::Frac(1)]) {
      Text(text: "left")
      #(ui! { Text(text: first) })
    }
  }
}

fn _build_iter_splice(rows: Vec<String>) -> Element {
  // Iterator splice of fresh Elements, each with its own `key` so
  // they get distinct ElementKeys when mounted (since they all
  // share the source span of the `ui!{}` inside the closure body
  // \u2014 user keys disambiguate them).
  let flex: Vec<Flex> = rows.iter().map(|_| Flex::Frac(1)).collect();
  ui! {
    VStack(.flex: flex) {
      #(rows.into_iter().enumerate().map(|(i, text)| ui! {
        Text(text: text, key: i.to_string())
      }))*
    }
  }
}

fn _build_mixed(extra: Vec<Element>, footer: Element) -> Element {
  ui! {
    VStack(.flex: [Flex::Frac(1)]) {
      Text(text: "header")
      #(extra.into_iter())*
      #footer
    }
  }
}

fn main() {}
