use std::collections::{HashMap, HashSet};

use crate::{components::Any as AnyComponent, render::ComponentID, utils::provider::Provider};

/// Components that have been rendered.
#[derive(Default)]
pub struct Components {
  /// The components that have been rendered.
  components: HashMap<ComponentID, AnyComponent>,
}

impl Components {
  /// Gets the component.
  #[must_use]
  pub fn get(&self, component_id: &ComponentID) -> Option<&AnyComponent> {
    self.components.get(component_id)
  }
}

impl Provider for Components {
  type Entry = AnyComponent;
  type Context = ();
  /// Components that were unmounted, to remove from the map.
  type Exit = HashSet<ComponentID>;
  type Output = ();

  fn enter(&mut self, component: &Self::Entry) -> Self::Context {
    self.components.insert(component.id, component.clone());
  }
  fn exit(&mut self, unmounted_component_ids: &Self::Exit) -> Self::Output {
    for component_id in unmounted_component_ids {
      self.components.remove(component_id);
    }
  }
}