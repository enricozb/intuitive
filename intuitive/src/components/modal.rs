//! Conditionally overlay a second child on top of a first.

use crate::{
  components::ZStack,
  macros::{component, ui},
  runtime::{Context, Element},
  signal::Signal,
};

/// Conditionally overlay a second child on top of a first.
///
/// The first composed child is the "base" and is always rendered; the second
/// composed child is the "overlay" and is rendered on top of the base only when
/// the `visible` signal is `true`. Internally this just composes a [`ZStack`],
/// so when both children render the second draws over the first.
///
/// Reading the `visible` signal subscribes the `Modal` element, so a
/// `Signal::set` / `Signal::update` that flips the value will cause `Modal` to
/// re-render and mount or unmount the overlay's subtree accordingly.
///
/// # Panics
///
/// On render, panics if the number of composed children is not exactly two.
#[component]
pub fn Modal(ctx: &mut Context<'_>, visible: Signal<bool>, #[children] children: Vec<Element>) -> Element {
  // Enforce exactly two children via the `Vec<T> -> [T; N]` conversion.
  let [base, overlay]: [Element; 2] = children
    .try_into()
    .unwrap_or_else(|v: Vec<Element>| panic!("Modal requires exactly two children, got {}", v.len()));

  ui! {
    ZStack() {
      #base
      if visible.get(ctx) { #overlay }
    }
  }
}
