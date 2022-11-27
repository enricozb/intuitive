mod component;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::render::component::Component;

pub fn parse(input: TokenStream) -> TokenStream {
  let component = parse_macro_input!(input as Component);

  quote! { #component }.into()
}
