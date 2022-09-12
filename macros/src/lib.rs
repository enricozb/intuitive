mod component;
mod on_key;
mod render;

mod utils;

use proc_macro::TokenStream;

/// Helper attribute macro for creating functional components.
///
/// See the documentation in the [`intuitive`] crate for details.
///
/// [`intuitive`]: https://docs.rs/intuitive/latest/intuitive/attr.component.html
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  component::parse(attr, item)
}

/// Macro for rendering components.
///
/// See the documentation in the [`intuitive`] crate for details.
///
/// [`intuitive`]: https://docs.rs/intuitive/latest/intuitive/macro.render.html
#[proc_macro]
pub fn render(item: TokenStream) -> TokenStream {
  render::parse(item)
}

/// Helper macro for creating key handlers.
///
/// See the documentation in the [`intuitive`] crate for details.
///
/// [`intuitive`]: https://docs.rs/intuitive/latest/intuitive/macro.on_key.html
#[proc_macro]
pub fn on_key(item: TokenStream) -> TokenStream {
  on_key::parse(item)
}
