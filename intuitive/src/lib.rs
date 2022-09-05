//! # Intuitive
//! Intuitive is a component-based library for creating text-based user interfaces
//! (TUIs) easily.
//!
//! TODO(enricozb): put a gif here?
//!
//! It is heavily inspired by [React](https://developer.apple.com/xcode/swiftui/) and
//! [SwiftUI](https://developer.apple.com/xcode/swiftui/), containing features that
//! resemble functional components, hooks, and a (mostly) declarative DSL.
//!
//! # Design
//! The main focus of Intuitive is to simplify the implementation of section-based TUIs,
//! such as [lazygit](https://github.com/jesseduffield/lazygit)'s. Resizing is handled
//! automatically, and keyboard events can be handled easily.
//!
//! For example, in order to roughly replicate lazygit's 7-panel layout, it would only
//! take the following code (ignoring `use` statements):
//! ```rust
//! #[component(Root)]
//! fn render() {
//!   render! {
//!     HStack(flex: [1, 2]) {
//!       VStack(flex: [Block(3), Grow(1), Grow(1), Grow(1), Block(3)]) {
//!         Section(title: "Status")
//!         Section(title: "Files - Submodules")
//!         Section(title: "Local Branches")
//!         Section(title: "Commits - Reflog")
//!         Section(title: "Stash")
//!       }
//!
//!       VStack(flex: [Grow(1), Block(10)]) {
//!         Section(title: "Unstaged Changes")
//!         Section(title: "Command Log")
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
//! TODO(enricozb): add image here

pub mod components;
pub mod element;
pub mod error;
pub mod event;
pub mod macros;
pub mod state;
pub mod terminal;

pub use intuitive_macros::component;
