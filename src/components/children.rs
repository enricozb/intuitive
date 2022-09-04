use std::ops::Deref;

use super::{element::Any as AnyElement, AnyComponent};

#[derive(Clone)]
pub struct Children<const N: usize>([AnyComponent; N]);

impl<const N: usize> Children<N> {
  pub fn render(&self) -> [AnyElement; N] {
    let mut components = [(); N].map(|()| AnyElement::default());

    for (i, component) in components.iter_mut().enumerate() {
      *component = self.0[i].render()
    }

    components
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
