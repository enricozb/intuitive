//! The [`Context`] type.

use std::collections::HashMap;

use super::Region;
use crate::{element::Any as AnyElement, error::Result, render::ComponentID};

/// The drawing context.
///
/// This keeps track of all inter-draw data in order to support:
/// - key and mouse event propagation
/// - focusing
#[derive(Default)]
pub struct Context {
  /// A stack of [`ComponentID`]'s.
  component_ids: Vec<ComponentID>,

  /// Maps an element to its parent.
  parents: HashMap<ComponentID, ComponentID>,
}

impl Context {
  /// Draws an element.
  pub fn draw<'a>(&mut self, element: &AnyElement, region: &'a mut Region<'a>) -> Result<()> {
    let component_id = if let Some(component_id) = element.id {
      component_id
    } else {
      return Ok(());
    };

    if let Some(parent_component_id) = self.component_ids.last() {
      self.parents.insert(component_id, *parent_component_id);
    }

    self.component_ids.push(component_id);

    element.draw(self, region)?;

    self.component_ids.pop();

    Ok(())
  }

  pub fn on_key(&mut self) {}
  pub fn on_mouse(&mut self) {}
}
