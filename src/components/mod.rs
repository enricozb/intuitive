pub mod any;
pub mod children;

mod centered;
mod embed;
mod empty;
mod modal;
mod section;
mod stack;
mod text;

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
use crate::{
  event::KeyEvent,
  terminal::{Frame, Rect},
};

pub trait Component {
  fn on_key(&self, _event: KeyEvent) {}

  fn render(&self) -> AnyComponent {
    Empty.into()
  }

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.render().draw(rect, frame);
  }
}
