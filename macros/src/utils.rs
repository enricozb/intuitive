use proc_macro2::{Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::Ident;

pub fn crate_name() -> TokenStream {
  let crate_name = proc_macro_crate::crate_name("intuitive").unwrap();

  match crate_name {
    FoundCrate::Itself => quote! { crate },
    FoundCrate::Name(name) => {
      let ident = Ident::new(&name, Span::call_site());
      quote! { #ident }
    }
  }
}
