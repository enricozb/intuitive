use std::collections::{HashMap, HashSet};

use crate::{render::ComponentID, utils::provider::Provider};

/// Descendants of elements with respect to rendering.
pub struct Descendants {
  /// A stack of component ids.
  component_ids: Vec<ComponentID>,

  /// The immediate descendants of a component with respect to rendering.
  descendants: HashMap<ComponentID, HashSet<ComponentID>>,
}

impl Descendants {
  /// Creates a new [`Descendants`].
  pub fn new() -> Self {
    Self {
      component_ids: Vec::new(),
      descendants: HashMap::new(),
    }
  }

  /// Returns all [`ComponentID`] descentants of `component_id`.
  fn descendants(&self, component_id: ComponentID) -> HashSet<ComponentID> {
    let mut descendants = HashSet::new();
    for component_id in self.descendants.get(&component_id).cloned().unwrap_or_default() {
      descendants.extend(self.descendants(component_id));
    }

    descendants
  }
}

impl Provider for Descendants {
  /// The component being rendered.
  type Entry = ComponentID;

  /// The old descendants for the given [`ComponentID`].
  type Context = HashSet<ComponentID>;
  type Exit = Self::Context;

  /// [`ComponentID`]s that are _no longer_ descendants [`ComponentID`] originally given to [`Self::entry`].
  type Output = HashSet<ComponentID>;

  fn enter(&mut self, component_id: &Self::Entry) -> Self::Context {
    // Add `component_id` to the descendants of `self.current_component_id`
    if let Some(parent_component_id) = self.component_ids.last() {
      self
        .descendants
        .get_mut(&parent_component_id)
        .map(|descendants| descendants.insert(*component_id));
    }

    // Remove the old descendants, in order to compare to them in [`Self::exit`].
    let old_descendants = self.descendants.remove(&component_id).unwrap_or_default();
    self.descendants.insert(*component_id, HashSet::new());

    self.component_ids.push(*component_id);

    old_descendants
  }

  fn exit(&mut self, old_descendants: &Self::Exit) -> Self::Output {
    let component_id = self.component_ids.pop().expect("pop");

    let new_descendants = self.descendants.get(&component_id).expect("get").clone();

    let mut unmounted_component_ids = HashSet::new();
    for unmounted_component_id in old_descendants.difference(&new_descendants).cloned() {
      unmounted_component_ids.insert(unmounted_component_id);
      unmounted_component_ids.extend(self.descendants(unmounted_component_id));
    }

    unmounted_component_ids
  }
}
