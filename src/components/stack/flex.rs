use std::ops::Deref;

#[derive(Clone, Copy)]
pub enum Flex {
  Block(u16),
  Grow(u16),
}

#[derive(Clone, Copy)]
pub struct Array<const N: usize> {
  flex: [Flex; N],
}

impl<const N: usize> From<[u16; N]> for Array<N> {
  fn from(flex: [u16; N]) -> Self {
    Self {
      flex: flex.map(Flex::Grow),
    }
  }
}

impl<const N: usize> From<[Flex; N]> for Array<N> {
  fn from(flex: [Flex; N]) -> Self {
    Self { flex }
  }
}

impl<const N: usize> Deref for Array<N> {
  type Target = [Flex; N];

  fn deref(&self) -> &Self::Target {
    &self.flex
  }
}

impl<const N: usize> Default for Array<N> {
  fn default() -> Self {
    Self { flex: [Flex::Grow(1); N] }
  }
}
