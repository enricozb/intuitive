use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  parse_macro_input, FnArg, Ident, ItemFn, Pat, PatType, Result, Token,
};

struct Name {
  component: Ident,
  crate_name: Ident,
}

impl Parse for Name {
  fn parse(input: ParseStream) -> Result<Self> {
    // if a component name begins with crate::, use `crate` instead of `intuitive` in paths.
    // this is needed for components that use this macro inside of `intuitive`
    let crate_name = if input.lookahead1().peek(Token![crate]) {
      input.parse::<Token![crate]>()?;
      input.parse::<Token![::]>()?;

      Ident::new("crate", Span::call_site())
    } else {
      Ident::new("intuitive", Span::call_site())
    };

    Ok(Self {
      component: input.parse()?,
      crate_name,
    })
  }
}

/// The implementation of the `component` attribute macro. See the
/// docs at the root of the crate for details.
pub fn parse(attr: TokenStream, item: TokenStream) -> TokenStream {
  let Name { component, crate_name } = parse_macro_input!(attr as Name);

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
      #(#params),*
    }

    impl #component {
      fn new(#(#params),*) -> #crate_name::components::Any {
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
