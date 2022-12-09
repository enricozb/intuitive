mod any;
mod embed;
mod empty;
mod fixed;
mod padding;
mod section;
mod stack;
mod text;

pub use self::{
  any::Any,
  embed::Embed,
  empty::Empty,
  fixed::Fixed,
  padding::Padding,
  section::Section,
  stack::{HStack, Stack, VStack},
  text::Text,
};
#[allow(unused)]
use crate::element::Element;
use crate::{element::Any as AnyElement, render::manager::Manager as RenderManager};

/// Describes types which can be rendered to an [`Element`].
pub trait Component: Default {
  fn render(&self, render: &mut RenderManager) -> AnyElement;
}
