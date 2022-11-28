use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  braced,
  parse::{Parse, ParseStream},
  token::Brace,
  Result,
};

use super::Component;
#[allow(unused)]
use crate::render;

/// The children passed to a component within [`render!`].
pub struct Children {
  /// The children.
  children: Vec<Component>,
}

impl Parse for Children {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut children = Vec::new();

    if input.peek(Brace) {
      let content;
      braced!(content in input);
      while !content.is_empty() {
        children.push(content.parse()?);
      }
    }

    Ok(Self { children })
  }
}

impl ToTokens for Children {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let children = &self.children;

    if !children.is_empty() {
      tokens.extend(quote! {
        children: [#(#children,)*].into(),
      });
    }
  }
}
