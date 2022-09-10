use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType};

use crate::utils;

/// The implementation of the `component` attribute macro. See the
/// docs at the root of the crate for details.
pub fn parse(attr: TokenStream, item: TokenStream) -> TokenStream {
  let component: Ident = syn::parse(attr).unwrap();
  let crate_name = utils::crate_name();

  let ItemFn {
    attrs, vis, sig, block, ..
  } = syn::parse(item).unwrap();
  let params: Vec<_> = sig.inputs.iter().collect();
  let param_names: Vec<Box<Pat>> = params
    .iter()
    .map(|input| match input {
      FnArg::Receiver { .. } => panic!("receivers not allowed in functional component"),
      FnArg::Typed(PatType { pat, .. }) => pat,
    })
    .cloned()
    .collect();

  quote! {
    #(#attrs)*
    #[derive(Default)]
    #vis struct #component {
      #(pub #params),*
    }

    impl #component {
      pub fn new(#(#params),*) -> #crate_name::components::Any {
        Self {
          #(#param_names),*
        }.into()
      }
    }

    impl #crate_name::components::Component for #component {
      fn render(&self) -> #crate_name::element::Any {
        let #component { #(#param_names),* } = self;

        #block
      }
    }
  }
  .into()
}
