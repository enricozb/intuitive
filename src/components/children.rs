use std::ops::Deref;

use super::AnyComponent;

#[derive(Clone)]
pub struct Children<const N: usize>([AnyComponent; N]);

impl<const N: usize> Children<N> {
  pub fn render(&self) -> Self {
    let mut components = [(); N].map(|()| AnyComponent::default());

    for i in 0..components.len() {
      components[i] = self.0[i].render()
    }

    Self(components)
  }
}

impl<const N: usize> Default for Children<N> {
  fn default() -> Self {
    Self([(); N].map(|_| AnyComponent::default()))
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
