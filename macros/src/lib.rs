mod render;
mod utils;

use proc_macro::TokenStream;

#[proc_macro]
pub fn render(item: TokenStream) -> TokenStream {
  render::parse(item)
}
