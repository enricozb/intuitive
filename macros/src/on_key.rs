use proc_macro::TokenStream;
use quote::quote;
use syn::{
  bracketed,
  parse::{Parse, ParseStream},
  parse_macro_input,
  punctuated::Punctuated,
  token::Bracket,
  Arm, Ident, Result, Token,
};

use crate::utils;

struct OnKey {
  capture: Punctuated<Ident, Token![,]>,
  arms: Vec<Arm>,
}

impl Parse for OnKey {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Bracket) {
      let content;
      bracketed!(content in input);

      Ok(Self {
        capture: content.parse_terminated(Ident::parse)?,
        arms: parse_arms(input)?,
      })
    } else {
      Ok(Self {
        capture: Punctuated::new(),
        arms: parse_arms(input)?,
      })
    }
  }
}

fn parse_arms(input: ParseStream) -> Result<Vec<Arm>> {
  let mut arms = Vec::new();
  while !input.is_empty() {
    // remove commas from arms, we add them later in `parse`
    let mut arm = input.call(Arm::parse)?;
    arm.comma = None;

    arms.push(arm);
  }

  Ok(arms)
}

/// The implementation of the `on_key` function-like macro. See the
/// docs at the root of the crate for details.
pub fn parse(input: TokenStream) -> TokenStream {
  let OnKey { capture, arms } = parse_macro_input!(input as OnKey);
  let arms = arms.iter();
  let capture = capture.iter();
  let crate_name = utils::crate_name();

  quote! {
    {
      #(let #capture = #capture.clone();)*

      move |event| {
        use #crate_name::event::{self, KeyCode::*, KeyEvent};

        match event {
          #(#arms,)*

          _ => (),
        }
      }
    }
  }
  .into()
}
