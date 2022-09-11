#![feature(doc_auto_cfg)]

//! # Intuitive
//! Intuitive is a component-based library for creating text-based user interfaces
//! (TUIs) easily.
//!
//! It is heavily inspired by [React] and [SwiftUI], containing features that
//! resemble functional components, hooks, and a (mostly) declarative DSL.
//!
//! Check out the [Getting Started] section below for a brief introduction to using Intuitive.
//!
//! # Design
//! The main focus of Intuitive is to simplify the implementation of section-based TUIs,
//! such as [lazygit](https://github.com/jesseduffield/lazygit)'s. Resizing is handled
//! automatically, and keyboard events can be handled easily.
//!
//! For example, a complex layout with an input box:
//! ```rust
//! use intuitive::{
//!   component,
//!   components::{stack::Flex::*, HStack, Section, Text, VStack},
//!   error::Result,
//!   on_key, render,
//!   state::use_state,
//!   terminal::Terminal,
//! };
//!
//! #[component(Root)]
//! fn render() {
//!   let text = use_state(|| String::new());
//!
//!   let on_key = on_key! { [text]
//!     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
//!     KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
//!     KeyEvent { code: Esc, .. } => event::quit(),
//!   };
//!
//!   render! {
//!     VStack(flex: [Block(3), Grow(1)], on_key) {
//!       Section(title: "Input") {
//!         Text(text: text.get())
//!       }
//!
//!       HStack(flex: [1, 2, 3]) {
//!         Section(title: "Column 1")
//!         Section(title: "Column 2")
//!         Section(title: "Column 3")
//!       }
//!     }
//!   }
//! }
//!
//! fn main() -> Result<()> {
//!   Terminal::new(Root::new())?.run()
//! }
//! ```
//! And the output would look like this:
//!
//! ![demo](https://raw.githubusercontent.com/enricozb/intuitive/main/assets/demo.png)
//!
//! # Getting Started
//! Similarly to [React], Intuitive is built around components that are composable.
//! There is one root component, that is passed to [`Terminal::new()`], in order to
//! run the TUI.
//!
//! There are two main ways to build components:
//! - Functional components using the [`component` attribute macro]
//! - Custom components by implementing [`Component`] and (potentially [`Element`])
//!
//! Both of these are discussed in depth in the [`components`] module documentation. Other
//! useful resources are:
//! - The [recipes] section of the [`components`] module documentation, describing ways to
//!   achieve common UI interactions.
//! - The [examples] directory in the repository, which contains complete examples of simple
//!   applications.
//!
//! # Disclaimer
//! Intuitive is really new and is missing some core features, such as a good story
//! around styling components, and coloring text in general. There may also be some
//! bugs in the library of components, please [raise an issue] if you find any.
//! Furthermore, since a large and complex application has yet to be built using
//! Intuitive, it is not a guarantee that it does not have some major flaw
//! making such development difficult.
//!
//! [raise an issue]: https://github.com/enricozb/intuitive/issues
//! [`component` attribute macro]: attr.component.html
//! [`Component`]: components/trait.Component.html
//! [`components`]: components/index.html
//! [`Element`]: element/trait.Element.html
//! [examples]: https://github.com/enricozb/intuitive/tree/main/examples
//! [Getting Started]: #getting-started
//! [React]: https://reactjs.org/
//! [recipes]: components/index.html#recipes
//! [SwiftUI]: https://developer.apple.com/xcode/swiftui/
//! [`Terminal::new()`]: terminal/struct.Terminal.html#method.new

pub mod components;
pub mod element;
pub mod error;
pub mod event;
pub mod spans;
pub mod state;
pub mod style;
pub mod terminal;

pub use intuitive_macros::{component, on_key, render};
