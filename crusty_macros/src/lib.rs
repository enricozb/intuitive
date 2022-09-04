use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn, Pat, PatType};

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  let component_name: Ident = syn::parse(attr).unwrap();
  let func: ItemFn = syn::parse(item).unwrap();

  let vis = func.vis;
  let fields = func.sig.inputs.iter();
  let params = func.sig.inputs.iter();
  let param_names: Vec<Box<Pat>> = func
    .sig
    .inputs
    .iter()
    .map(|input| match input {
      FnArg::Receiver { .. } => panic!("receivers not allowed in functional component"),
      FnArg::Typed(PatType { pat, .. }) => pat,
    })
    .cloned()
    .collect();

  let props = param_names.clone();

  let block = func.block;

  quote! {
    #[derive(Default)]
    #vis struct #component_name {
      #(#fields),*
    }

    impl #component_name {
      fn new(#(#params),*) -> crusty::components::AnyComponent {
        Self {
          #(#param_names),*
        }.into()
      }
    }

    impl crusty::components::Component for #component_name {
      fn render(&self) -> crusty::element::Any {
        let #component_name { #(#props),* } = self;

        #block
      }
    }
  }
  .into()
}
