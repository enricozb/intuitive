use crate::{
  component,
  element::{Any as AnyElement, Element},
};

/// A component that renders nothing.
#[component(crate::Empty)]
pub fn render() -> AnyElement {
  AnyElement::new(Self {})
}

impl Element for Empty {}
