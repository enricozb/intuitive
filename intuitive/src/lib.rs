// A hack so macro uses within intuitive use the correct crate name.
extern crate self as intuitive;

pub mod buffer;
pub mod components;
pub mod element;
pub mod error;
pub mod render;
pub mod style;
pub mod terminal;
pub mod utils;

pub mod event;

pub use intuitive_macros::{component, render};
