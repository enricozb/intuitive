pub mod any;
pub mod children;
pub mod modal;

mod centered;
mod embed;
mod empty;
mod section;
mod stack;
mod table;
mod text;

pub use self::{
  any::Any as AnyComponent,
  centered::Centered,
  embed::Embed,
  empty::Empty,
  section::Section,
  stack::{Flex, Horizontal as HStack, Vertical as VStack},
  table::Table,
  text::Text,
};
use crate::element::Any as AnyElement;

pub trait Component {
  fn render(&self) -> AnyElement {
    Empty.render()
  }
}
