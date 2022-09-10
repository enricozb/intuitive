#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
  Left,
  Right,
}

impl Default for Alignment {
  fn default() -> Self {
    Self::Left
  }
}

#[derive(Clone, Copy)]
pub struct Array<const N: usize> {
  alignments: [Alignment; N],
}

impl<const N: usize> Default for Array<N> {
  fn default() -> Self {
    Self {
      alignments: [(); N].map(|_| Alignment::default()),
    }
  }
}

impl<const N: usize> From<Array<N>> for [Alignment; N] {
  fn from(array: Array<N>) -> Self {
    array.alignments
  }
}

impl<const N: usize> From<[Alignment; N]> for Array<N> {
  fn from(alignments: [Alignment; N]) -> Self {
    Self { alignments }
  }
}
