use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  braced, parenthesized,
  parse::{Parse, ParseStream},
  parse_macro_input,
  punctuated::Punctuated,
  token::Brace,
  Expr, Ident, Result, Token,
};

struct Component {
  name: Ident,
  params: Punctuated<Param, Token![,]>,
  children: Vec<Component>,
}

impl ToTokens for Component {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let Self { name, params, children } = self;
    let params = params.iter();

    let children = if children.is_empty() {
      quote! {}
    } else {
      quote! {
        children: [#(#children,)*].into(),
      }
    };

    tokens.extend(quote! {
      #name {
        #(#params,)*

        #children

        ..Default::default()
      }.into()
    });
  }
}

impl Parse for Component {
  fn parse(input: ParseStream) -> Result<Self> {
    let name = input.parse()?;
    let params;
    parenthesized!(params in input);
    let params = params.parse_terminated(Param::parse)?;

    let mut children = Vec::new();

    if input.lookahead1().peek(Brace) {
      let content;
      braced!(content in input);
      while !content.is_empty() {
        children.push(content.parse()?);
      }
    }

    Ok(Self { name, params, children })
  }
}

enum Param {
  Field(Ident),
  Pair(Ident, Expr),
}

impl ToTokens for Param {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let (ident, value) = match self {
      Self::Field(ident) => (ident, quote! { #ident }),
      Self::Pair(ident, expr) => (ident, quote! { #expr }),
    };

    tokens.extend(quote! {
      #ident: #value
        .try_into()
        .expect(&format!("try into failed for argument: '{}'", stringify!(#ident)))
    });
  }
}

impl Parse for Param {
  fn parse(input: ParseStream) -> Result<Self> {
    let ident: Ident = input.parse()?;
    if input.lookahead1().peek(Token![,]) {
      Ok(Self::Field(ident))
    } else {
      input.parse::<Token![:]>()?;

      Ok(Self::Pair(ident, input.parse()?))
    }
  }
}

/// The implementation of the `render` function-like macro. See the
/// docs at the root of the crate for details.
pub fn parse(input: TokenStream) -> TokenStream {
  let component = parse_macro_input!(input as Component);

  quote! {
    #component
  }
  .into()
}
