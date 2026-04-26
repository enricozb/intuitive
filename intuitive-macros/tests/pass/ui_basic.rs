//! End-to-end pass test: a `ui!{}` invocation matching the example
//! from `macros.md` compiles and produces a value of type `Element`.

use intuitive::{
  components::{HStack, Text},
  geometry::Flex,
  macros::ui,
  runtime::Element,
};

fn _build(some_bool: bool) -> Element {
  // Demonstrates the `Into`-driven sugar at every prop slot:
  // `[..]` for the `flex: Vec<Flex>` required arg, and bare
  // `&'static str` literals for the `text: String` required arg.
  ui! {
    HStack(.flex: [Flex::Frac(1), Flex::Frac(1)]) {
      Text(text: "a")
      if some_bool {
        Text(text: "b")
      } else {
        Text(text: "c")
      }
    }
  }
}

fn main() {}
