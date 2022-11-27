use syn::{
  braced,
  parse::{Parse, ParseStream},
  token::Brace,
  Result,
};

use super::Component;

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
