//! `if` at the top of `ui!{}` is not allowed: there's no surrounding
//! `Vec<Element>` to push into, and the macro must produce a single
//! `Element` value.

use intuitive::macros::ui;

fn main() {
  let _ = ui! {
    if true { a() } else { b() }
  };
}
