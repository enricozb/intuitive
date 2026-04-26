//! End-to-end pass test: the no-parens shortcut form for components
//! whose required props are all defaulted (or absent) compiles and
//! produces a value of type `Element`. `ZStack { .. }` is sugar for
//! `ZStack() { .. }`, and `Empty` (a true leaf with no args at all)
//! is sugar for `Empty()`.

use intuitive::{
  components::{Empty, ZStack},
  macros::ui,
  runtime::Element,
};

fn _stack() -> Element {
  // `ZStack` has no required non-`#[children]` props, so the
  // parens are pure noise: `ZStack { .. }` must work.
  ui! {
    ZStack {
      Empty
      Empty()
    }
  }
}

fn _leaf() -> Element {
  // `Empty` has no args at all, so all four spellings are
  // equivalent. `Empty` alone (bare ident, no parens, no children)
  // is the most compact form.
  ui! { Empty }
}

fn main() {}
