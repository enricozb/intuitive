//! Generic array type imlementing common convenience traits.

use std::ops::Deref;

/// An array type that implements common convenience traits that are not yet implemented for `[T; N]` over all `N`.
pub struct Array<const N: usize, T>(pub [T; N]);

impl<const N: usize, T> From<[T; N]> for Array<N, T> {
  fn from(array: [T; N]) -> Self {
    Self(array)
  }
}

impl<const N: usize, T> Deref for Array<N, T> {
  type Target = [T; N];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<const N: usize, T> Clone for Array<N, T>
where
  T: Clone,
{
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<const N: usize, T> Default for Array<N, T>
where
  T: Default,
{
  fn default() -> Self {
    Self([(); N].map(|()| T::default()))
  }
}
