mod component;
mod render;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  component::parse(attr, item)
}

#[proc_macro]
pub fn render(item: TokenStream) -> TokenStream {
  render::parse(item)
}
