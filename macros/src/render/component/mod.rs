mod children;
mod params;

use std::sync::atomic::{AtomicUsize, Ordering};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  parse::{Parse, ParseStream},
  Ident, Result,
};

use self::{children::Children, params::Params};
#[allow(unused)]
use crate::render;
use crate::utils;

/// An atomic counter used to create unique integer ids to every instance of a
/// component within a [`render!`] invocation.
static COMPONENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A component within [`render!`].
pub struct Component {
  /// A unique id. This is set automatically by [`Component::parse`].
  uid: usize,

  /// The name of the component,
  name: Ident,

  /// The parameters passed to the component.
  params: Params,

  /// The children passed to the component.
  children: Children,
}

impl Parse for Component {
  fn parse(input: ParseStream) -> Result<Self> {
    let uid = COMPONENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

    Ok(Self {
      uid,
      name: input.parse()?,
      params: input.parse()?,
      children: input.parse()?,
    })
  }
}

impl ToTokens for Component {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let crate_name = utils::crate_name();
    let component_id = self.component_id();

    let Self {
      name, params, children, ..
    } = self;

    tokens.extend(quote! {
      #crate_name::render::render(
        #component_id,
        #name {
          #params,
          #children,
          ..Default::default(),
        },
      )
    });
  }
}

impl Component {
  /// A unique component identifier. Used during rendering.
  fn component_id(&self) -> TokenStream2 {
    let Self { uid, name, params, .. } = self;

    let key = match &params.key {
      Some(key) => quote! { Some(#key) },
      None => quote! { None },
    };

    let crate_name = utils::crate_name();

    quote! {
      #crate_name::render::ComponentID {
        name: stringify!(#name),
        key: #key,
        file: file!(),
        uid: #uid,
      }
    }
  }
}
