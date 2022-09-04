use crate::components::{
  element::{Any as AnyElement, Element},
  Component,
};

#[derive(Clone, Default)]
pub struct Empty;

impl Component for Empty {
  fn render(&self) -> AnyElement {
    AnyElement::new(Empty)
  }
}

impl Element for Empty {}
