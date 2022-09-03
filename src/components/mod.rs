pub mod any;
pub mod children;

mod centered;
mod embed;
mod empty;
mod section;
mod stack;
mod text;

pub use self::{
  any::Any as AnyComponent,
  centered::Centered,
  embed::Embed,
  empty::Empty,
  section::Section,
  stack::{Flex, Horizontal as HStack, Vertical as VStack},
  text::Text,
};
use crate::{
  event::KeyEvent,
  terminal::{Frame, Rect},
};

pub trait Component {
  fn on_key(&self, event: KeyEvent) {
    self.render().on_key(event);
  }

  fn render(&self) -> AnyComponent;

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.render().draw(rect, frame);
  }
}
