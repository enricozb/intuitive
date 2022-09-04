pub mod any;
pub mod children;

mod centered;
mod embed;
mod empty;
mod modal;
mod section;
mod stack;
mod text;

pub use crusty_macros::component;

pub use self::{
  any::Any as AnyComponent,
  centered::Centered,
  embed::Embed,
  empty::Empty,
  modal::{use_modal_funcs, Modal},
  section::Section,
  stack::{Flex, Horizontal as HStack, Vertical as VStack},
  text::Text,
};
use crate::element::Any as AnyElement;

pub trait Component {
  fn render(&self) -> AnyElement {
    Empty.render()
  }
}
