use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType};

/// The implementation of the `component` attribute macro. See the
/// docs at the root of the crate for details.
pub fn parse(attr: TokenStream, item: TokenStream) -> TokenStream {
  let component_name: Ident = syn::parse(attr).unwrap();

  let ItemFn { vis, sig, block, .. } = syn::parse(item).unwrap();
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
    #[derive(Default)]
    #vis struct #component_name {
      #(#params),*
    }

    impl #component_name {
      fn new(#(#params),*) -> intuitive::components::Any {
        Self {
          #(#param_names),*
        }.into()
      }
    }

    impl intuitive::components::Component for #component_name {
      fn render(&self) -> intuitive::element::Any {
        let #component_name { #(#param_names),* } = self;

        #block
      }
    }
  }
  .into()
}
