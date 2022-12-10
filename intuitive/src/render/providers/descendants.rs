use std::collections::{HashMap, HashSet};

use crate::{render::ComponentID, utils::provider::Provider};

pub struct Descendants {
  /// The parent [`ComponentID`], `None` if this is the first render.
  parent_component_id: Option<ComponentID>,

  /// The immediate descendants of a component with respect to rendering.
  descendants: HashMap<ComponentID, HashSet<ComponentID>>,
}

impl Descendants {
  /// Creates a new [`Descendants`].
  pub fn new() -> Self {
    Self {
      parent_component_id: None,
      descendants: HashMap::new(),
    }
  }

  /// Returns all [`ComponentID`] descentants of `component_id`.
  fn descendants(&self, component_id: ComponentID) -> HashSet<ComponentID> {
    let descendants = HashSet::new();
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

  /// The component being rendered.
  type Exit = ComponentID;

  /// [`ComponentID`]s that are _no longer_ descendants [`ComponentID`] originally given to [`Self::entry`].
  type Output = HashSet<ComponentID>;

  fn enter(&mut self, component_id: Self::Entry) -> Self::Context {
    // Add `component_id` to the descendants of `self.current_component_id`
    if let Some(parent_component_id) = self.parent_component_id {
      self
        .descendants
        .get_mut(&parent_component_id)
        .map(|descendants| descendants.insert(component_id));
    }

    // Remove the old descendants, in order to compare to them in [`Self::exit`].
    let old_descendants = self.descendants.remove(&component_id).unwrap_or_default();
    self.descendants.insert(component_id, HashSet::new());

    self.parent_component_id = Some(component_id);

    old_descendants
  }

  fn exit(&mut self, old_descendants: Self::Context, component_id: Self::Exit) -> Self::Output {
    let new_descendants = self.descendants.get(&component_id).expect("DESCENDANTS::get").clone();

    let unmounted_component_ids = HashSet::new();
    for unmounted_component_id in old_descendants.difference(&new_descendants).cloned() {
      unmounted_component_ids.extend(self.descendants(unmounted_component_id));
    }

    unmounted_component_ids
  }
}
