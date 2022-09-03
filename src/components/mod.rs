mod empty;
mod section;
mod text;

use std::{ops::Deref, rc::Rc};

pub use self::{empty::Empty, section::Section, text::Text};
use crate::{
  event::KeyEvent,
  terminal::{Frame, Rect},
};

#[derive(Clone)]
pub struct Any(Rc<dyn Component>);

impl Any {
  fn new<C: Component + 'static>(component: C) -> Self {
    Any(Rc::new(component))
  }
}

impl Default for Any {
  fn default() -> Self {
    Empty.into()
  }
}

impl Deref for Any {
  type Target = Rc<dyn Component>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<C: Component + 'static> From<C> for Any {
  fn from(component: C) -> Self {
    Self::new(component)
  }
}

pub trait Component {
  fn on_key(&self, event: KeyEvent) {
    self.render().on_key(event);
  }

  fn render(&self) -> Any;

  fn draw(&self, rect: Rect, frame: &mut Frame) {
    self.render().draw(rect, frame);
  }
}
