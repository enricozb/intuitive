use crate::{
  components::Component,
  element::{Any as AnyElement, Element},
};

#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self) -> AnyElement {
    AnyElement::new(Empty)
  }
}

impl Element for Empty {}
