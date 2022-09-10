use std::ops::Deref;

/// Control how much each child of a stack component grows
///
/// For example:
/// ```rust
/// render! {
///   VStack(flex: [1, 2, 3]) {
///     Section(title: "small")
///     Section(title: "medium")
///     Section(title: "large")
///   }
/// }
/// ```
/// will render a vertical stack of three [`Section`] components. The bottom one
/// will be 3 times the height of the top one, and the middle one will be 2 times the height
/// of the top one, as shown in the [`VStack`] docs.
///
/// When using the `flex` parameter to [`VStack`] and [`HStack`], providing a value
/// of type `[u16; N]`, will assume that [`Flex::Grow`] is intended, therefore making
/// all dimensions relative. In order to have absolute height or width for a child, provide a
/// value of type `[Flex; N]` to the `flex` parameter. For example,
/// ```rust
/// render! {
///   VStack(flex: [Block(3), Grow(1), Block(3)]) {
///     Section(title: "absolute")
///     Section(title: "relative")
///     Section(title: "absolute")
///   }
/// }
/// ```
///
/// [`HStack`]: ../struct.HStack.html
/// [`Section`]: ../struct.Section.html
/// [`VStack`]: ../struct.VStack.html
/// [`Flex::Grow`]: #variant.Grow
#[derive(Clone, Copy)]
pub enum Flex {
  /// An absolute amount of height or width.
  Block(u16),
  /// A relative amount of height or width.
  Grow(u16),
}

/// An array of [`Flex`] values.
///
/// This struct exists in order to implement `From<[Flex; N]>` and
/// `From<[u16; N]>`.
///
/// [`Flex`]: enum.Flex.html
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
