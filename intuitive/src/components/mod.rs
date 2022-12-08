mod any;
mod embed;
mod empty;
mod fixed;
mod padding;
mod section;
mod stack;
mod text;

pub use embed::Embed;
pub use empty::Empty;
pub use fixed::Fixed;
pub use padding::Padding;
pub use section::Section;
pub use stack::{HStack, Stack, VStack};
pub use text::Text;

pub(crate) use self::any::Any;
#[allow(unused)]
use crate::element::Element;
use crate::{element::Any as AnyElement, render::Manager as RenderManager};

/// Describes types which can be rendered to an [`Element`].
pub trait Component: Default {
  fn render(&self, render: &mut RenderManager) -> AnyElement;
}
