//! Structures relating to the `HStack` and `VStack` components.

mod flex;
pub(super) mod horizontal;
pub(super) mod vertical;

pub use self::flex::{Array as FlexArray, Flex};
