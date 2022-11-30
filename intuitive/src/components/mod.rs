mod any;
mod padding;
mod section;
mod text;

pub use padding::Padding;
pub use section::Section;
pub use text::Text;

pub use self::any::Any;
use crate::element::Any as AnyElement;
#[allow(unused)]
use crate::element::Element;

/// Describes types which can be rendered to an [`Element`].
pub trait Component: Default {
  fn render(&self) -> AnyElement;
}
