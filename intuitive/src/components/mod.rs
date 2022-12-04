mod any;
mod embed;
mod fixed;
mod padding;
mod section;
mod stack;
mod text;

pub use embed::Embed;
pub use fixed::Fixed;
pub use padding::Padding;
pub use section::Section;
pub use stack::{HStack, Stack, VStack};
pub use text::Text;

pub(crate) use self::any::Any;
use crate::element::Any as AnyElement;
#[allow(unused)]
use crate::element::Element;

/// Describes types which can be rendered to an [`Element`].
pub trait Component: Default {
  fn render(&self) -> AnyElement;
}
