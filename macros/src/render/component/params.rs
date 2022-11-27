use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  parenthesized,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  Error, Expr, Ident, Result, Token,
};

/// The parameters passed to a component within [`render!`].
pub struct Params {
  /// The `key` parameter, if one was provided.
  pub key: Option<TokenStream2>,

  /// The non-`key` parameters.
  params: Vec<Param>,
}

impl Parse for Params {
  fn parse(input: ParseStream) -> Result<Self> {
    let params;
    parenthesized!(params in input);

    let params: Punctuated<Param, Token![,]> = params.parse_terminated(Param::parse)?;
    let (keys, params): (Vec<_>, Vec<_>) = params.into_iter().partition(Param::is_key);

    if keys.len() > 1 {
      let extra_key = keys.get(1).unwrap();
      return Err(Error::new(extra_key.name().span(), "field `key` specified more than once"));
    }

    let key = match keys.get(0) {
      Some(Param::Short(key)) => Some(quote! { #key }),
      Some(Param::Long(_, expr)) => Some(quote! { #expr }),
      None => None,
    };

    Ok(Self { key, params })
  }
}

impl ToTokens for Params {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let params = &self.params;

    tokens.extend(quote! {
      #(#params),*
    });
  }
}

/// A single parameter passed to a component within [`render!`].
pub enum Param {
  /// Long-form syntax for passing a parameter. For example,
  /// ```
  /// render! {
  ///   Text(text: "Hello, world!")
  /// }
  /// ```
  Long(Ident, Box<Expr>),

  /// Shorthand syntax for passing a parameter. For example,
  /// ```
  /// let text = "Hello, world!";
  ///
  /// render! {
  ///   Text(text)
  /// }
  /// ```
  Short(Ident),
}

impl Parse for Param {
  fn parse(input: ParseStream) -> Result<Self> {
    let ident: Ident = input.parse()?;

    if input.peek(Token![:]) {
      input.parse::<Token![:]>()?;
      Ok(Self::Long(ident, input.parse()?))
    } else {
      Ok(Self::Short(ident))
    }
  }
}

impl ToTokens for Param {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let param = match self {
      Self::Short(name) => quote! { #name: #name},
      Self::Long(name, expr) => quote! { #name: #expr },
    };

    tokens.extend(quote! {
      #param
        .try_into()
        .expect(&format!("try into failed for argument: '{}'", stringify!(#param)))
    });
  }
}

impl Param {
  /// The parameter name.
  pub fn name(&self) -> &Ident {
    match self {
      Self::Short(name) | Self::Long(name, _) => name,
    }
  }

  /// Whether the parameter's name is `key`.
  pub fn is_key(&self) -> bool {
    self.name() == "key"
  }
}
