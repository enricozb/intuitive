//! A component-based library for building text user interfaces.
//!
//! # Getting Started & Quick Links
//!
//! - [Macros reference] for the [`component`] and [`ui!`] macros.
//! - [Components reference] for the built-in collection of components.
//!
//! # Demo
//!
//! ```
//! # use intuitive::buffer::{Color::{Green, Red, Yellow}, Decoration::Bold, Style};
//! # use intuitive::components::{Border, Centered, HStack, Text, VStack};
//! # use intuitive::geometry::{Align::Center, Flex::{Cell, Frac}};
//! # use intuitive::macros::{component, ui};
//! # use intuitive::runtime::{Context, Element};
//! # use intuitive::signal::Signal;
//! #[component]
//! fn Root(ctx: &mut Context<'_>) -> Element {
//!   let clicks = ctx.new_signal(0);
//!   let clicks_now = clicks.get(ctx);
//!   ui! {
//!     VStack(.flex: [Cell(3), Frac(1)]) {
//!       Border(.title: "Usage", .style: Red) {
//!         Text(text: "Click the clicker to increment, Esc to quit", .align: Center)
//!       }
//!       HStack {
//!         Centered(.height: Cell(3), .width: Frac(2)) {
//!             Clicker(clicks, .style: Bold)
//!         }
//!         Border(.title: "Counter", .style: Yellow) {
//!           Text(text: format!("{clicks_now} clicks"), .align: Center)
//!         }
//!       }
//!     }
//!   }
//! }
//!
//! #[component]
//! fn Clicker(ctx: &mut Context<'_>, clicks: Signal<i32>, #[default] style: Style) -> Element {
//!   ctx.on_click({
//!     let clicks = clicks.clone();
//!     move |_| clicks.update(|c| *c += 1)
//!   });
//!   ui! {
//!     Border(.style: Green) {
//!       Text(text: "Click me!", .style, .align: Center)
//!     }
//!   }
//! }
//! ```
//!
//! [Macros reference]: macros/
//! [Components reference]: components/
//! [`component`]: intuitive::macros::component
//! [`ui!`]: intuitive::macros::ui

// The proc macros in `intuitive-macros` emit fully-qualified paths
// like `::intuitive::runtime::Element`. When the `intuitive` crate
// compiles itself, the crate name `intuitive` would otherwise not
// resolve. `extern crate self as intuitive` makes our own crate
// reachable under its public name, so the same emitted paths work
// both inside this crate and in downstream user crates.
extern crate self as intuitive;

pub mod buffer;
pub mod components;
pub mod event;
pub mod geometry;
pub mod macros;
pub mod runtime;
pub mod signal;
pub mod terminal;
