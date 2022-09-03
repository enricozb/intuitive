use std::ops::Deref;

use super::AnyComponent;

#[derive(Clone)]
pub struct Children<const N: usize>([AnyComponent; N]);

impl<const N: usize> Default for Children<N> {
  fn default() -> Self {
    Self([(); N].map(|_| Default::default()))
  }
}

impl<const N: usize> Deref for Children<N> {
  type Target = [AnyComponent; N];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<const N: usize> From<[AnyComponent; N]> for Children<N> {
  fn from(children: [AnyComponent; N]) -> Self {
    Self(children)
  }
}
