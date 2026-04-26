//! A collection of simple reusable components.
//!
//! Every component in this module is defined via the [`component`] attribute
//! macro and is usually referenced through the [`ui!`] macro:
//!
//! ```
//! use intuitive::{components::{HStack, Text}, geometry::Flex::*, macros::ui};
//!
//! let stack = ui! {
//!   HStack(.flex: [Frac(1), Frac(2)]) {
//!     Text(text: "Hello,")
//!     Text(text: "World!")
//!   }
//! };
//! ```
//!
//! Because of how the [`component`] and [`ui!`] macros work, throughout the
//! generated documentation, every component will be a struct with generic
//! public fields. These structs are the _required_ properties of the
//! components. Optional properties are methods on a corresponding `*Builder`
//! struct.
//!
//! See the documentation in the [`macros`] module for details.
//!
//! [`macros`]: intuitive::macros
//! [`component`]: intuitive::macros::component
//! [`ui!`]: intuitive::macros::ui

pub mod border;
pub mod empty;
pub mod input;
pub mod layout;
pub mod list;
pub mod modal;
pub mod text;

pub use border::Border;
pub use empty::Empty;
pub use input::Input;
pub use layout::{Centered, HStack, VStack, ZStack};
pub use list::List;
pub use modal::Modal;
pub use text::Text;
