use std::collections::HashMap;

use crate::{element::Any as AnyElement, render::ComponentID, utils::provider::Provider};

/// Elements that have been rendered.
#[derive(Default)]
pub struct Elements {
  /// The elements that have been rendered.
  elements: HashMap<ComponentID, AnyElement>,
}

pub struct Exit {
  pub component_id: ComponentID,
  pub element: AnyElement,
  pub is_rerender: bool,
}

impl Provider for Elements {
  type Entry = ();
  type Context = ();
  type Exit = Exit;
  type Output = ();

  fn enter(&mut self, (): &Self::Entry) -> Self::Context {}

  fn exit(&mut self, exit: &Self::Exit) -> Self::Output {
    if exit.is_rerender {
      if let Some(old_element) = self.elements.get(&exit.component_id) {
        old_element.swap(&exit.element);
      }
    } else {
      self.elements.insert(exit.component_id, exit.element.clone());
    }
  }
}