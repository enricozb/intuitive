pub mod any;
pub mod children;
pub mod element;

mod centered;
mod embed;
mod empty;
mod modal;
mod section;
mod stack;
mod text;

use element::Any as AnyElement;

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

pub trait Component {
  fn render(&self) -> AnyElement {
    Empty.render()
  }
}
