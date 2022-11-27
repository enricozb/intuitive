mod children;
mod params;

use syn::{
  parse::{Parse, ParseStream},
  Ident, Result,
};

use self::{children::Children, params::Params};

/// A component within [`render!`].
pub struct Component {
  /// The name of the component,
  name: Ident,

  /// The parameters passed to the component.
  params: Params,

  /// The children passed to the component.
  children: Children,
}

impl Parse for Component {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Self {
      name: input.parse()?,
      params: input.parse()?,
      children: input.parse()?,
    })
  }
}
