mod component;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{render::component::Component, utils};

pub fn parse(input: TokenStream) -> TokenStream {
  let crate_name = utils::crate_name();

  let component = parse_macro_input!(input as Component);
  let component_id = component.component_id();

  quote! { #crate_name::render::render(#component_id, #component) }.into()
}
