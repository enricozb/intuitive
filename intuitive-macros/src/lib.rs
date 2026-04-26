//! Procedural macros ([`component`] and [`ui!`]) for the [`intuitive`] crate.
//!
//! You probably should be reading the [`macro` module-level documentation in
//! the `intuitive`][0] crate for documentation on using these macros.
//!
//! This crate is a thin shim: each entry point converts the
//! [`proc_macro::TokenStream`] into a [`proc_macro2::TokenStream`] and
//! dispatches to [`intuitive_macros_core`], converting the result back.
//! This split keeps [`intuitive_macros_core`] unit-testable.
//!
//! [`intuitive`]: /intuitive
//! [0]: /intuitive/macros

use proc_macro::TokenStream;

/// Build an `Element` from a declarative tree.
///
/// See the [`macro` module-level documentation in the `intuitive`][0] crate for
/// details.
///
/// [0]: /intuitive/macros
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
  match intuitive_macros_core::expand_ui(input.into()) {
    Ok(ts) => ts.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

/// Defines a functional component.
///
/// See the [`macro` module-level documentation in the `intuitive`][0] crate for
/// details.
///
/// [0]: /intuitive/macros
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  match intuitive_macros_core::expand_component(attr.into(), item.into()) {
    Ok(ts) => ts.into(),
    Err(e) => e.to_compile_error().into(),
  }
}
