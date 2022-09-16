use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, FnArg, Generics, Ident, ItemFn, Pat, PatType, Result, ReturnType,
};

use crate::utils;

pub struct Component {
  name: Ident,
  generics: Generics,
}

impl Parse for Component {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Self {
      name: input.parse()?,
      generics: input.parse()?,
    })
  }
}

/// The implementation of the `component` attribute macro. See the
/// docs at the root of the crate for details.
pub fn parse(attr: TokenStream, item: TokenStream) -> TokenStream {
  let Component { name, generics } = parse_macro_input!(attr as Component);
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let crate_name = utils::crate_name();

  let ItemFn { attrs, vis, sig, block } = syn::parse(item).unwrap();
  let params: Vec<_> = sig.inputs.iter().collect();

  let retty = match sig.output {
    ReturnType::Default => syn::parse(quote! { -> #crate_name::element::Any }.into()).unwrap(),
    retty => retty,
  };

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
    #vis struct #name #generics {
      #(pub #params),*
    }

    impl #impl_generics #name #ty_generics #where_clause {
      pub fn new(#(#params),*) -> #crate_name::components::Any {
        Self {
          #(#param_names),*
        }.into()
      }
    }

    impl #impl_generics #crate_name::components::Component for #name #ty_generics #where_clause {
      fn render(&self) #retty {
        let #name{ #(#param_names),* } = self;

        #block
      }
    }
  }
  .into()
}
