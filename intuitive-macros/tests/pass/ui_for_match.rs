//! End-to-end pass test: `for` and `match` in child position.

use intuitive::{
  components::{Text, VStack},
  geometry::Flex,
  macros::ui,
  runtime::Element,
};

enum Mode {
  A,
  B,
}

fn _build_for(tasks: Vec<String>) -> Element {
  let flex = vec![Flex::Frac(1); tasks.len()];
  ui! {
    VStack(.flex: flex) {
      for t in tasks.clone() {
        Text(text: t)
      }
    }
  }
}

fn _build_match(mode: Mode) -> Element {
  ui! {
    VStack(.flex: [Flex::Frac(1)]) {
      match mode {
        Mode::A => Text(text: "a"),
        Mode::B => { Text(text: "b") }
      }
    }
  }
}

fn main() {}
