//! End-to-end pass test: field shorthand in `ui!{}` argument lists.
//! `Foo(bar)` is sugar for `Foo(bar: bar)`, and `.foo` is sugar for
//! `.foo: foo`, mirroring Rust's struct field shorthand.

use intuitive::{
  components::{Text, ZStack},
  geometry::Align,
  macros::ui,
  runtime::Element,
};

fn _shorthand(text: String, align: Align) -> Element {
  // Required shorthand: `text` instead of `text: text`.
  // Optional shorthand: `.align` instead of `.align: align`.
  ui! { Text(text, .align) }
}

fn _mixed(text: String) -> Element {
  // Shorthand and explicit forms mixed in one arg list.
  ui! { Text(text, .align: Align::Center) }
}

fn _in_children(text: String) -> Element {
  // Shorthand also works for children inside a container.
  ui! {
    ZStack {
      Text(text)
    }
  }
}

fn main() {}
