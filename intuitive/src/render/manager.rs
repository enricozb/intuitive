use std::collections::{HashMap, HashSet};

use crate::{
  components::{Any as AnyComponent, Component},
  element::Any as AnyElement,
  error::{Error, Result},
  render::{hooks::manager::Manager as HookManager, ComponentID},
};

/// Manages rendering.
pub struct Manager {
  /// Maps a component ID to its component.
  components: HashMap<ComponentID, AnyComponent>,

  /// Maps a component to the element it most recently returned after a render.
  elements: HashMap<ComponentID, AnyElement>,

  /// Maps a component to the components it renders directly..
  descendants: HashMap<ComponentID, HashSet<ComponentID>>,

  /// Manages the hooks.
  pub hooks: HookManager,
}

impl Manager {
  /// Creates a new [`Manager`].
  #[must_use]
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
      elements: HashMap::new(),
      descendants: HashMap::new(),
      hooks: HookManager::new(),
    }
  }

  /// Renders a component.
  pub fn render(&mut self, component: AnyComponent) -> AnyElement {
    let component_id = component.id;
    let element = self.render_impl(&component);

    self.components.insert(component_id, component);
    self.elements.insert(component_id, element.clone());

    element
  }

  /// Re-renders an already rendered component, specified by its [`ComponentID`].
  ///
  /// # Errors
  ///
  /// Will return `Err` if a component has not yet been rendered with the provided [`ComponentID`].
  pub fn rerender(&mut self, component_id: ComponentID) -> Result<()> {
    if let Some(component) = self.components.get(&component_id) {
      let old_element = self.elements.get(&component_id).ok_or(Error::NoElement(component_id))?.clone();

      old_element.swap(&self.render_impl(&component.clone()));
    }

    Ok(())
  }

  /// Renders the root component, which does not have a hard-coded [`ComponentID`].
  pub(crate) fn render_root<C: Component + 'static>(&mut self, component: C) -> AnyElement {
    let component = AnyComponent::new(ComponentID::ROOT, component);

    self.render(component)
  }

  /// Renders a component, unmounting any elements if necessary.
  fn render_impl(&mut self, component: &AnyComponent) -> AnyElement {
    if let Some(parent_component_id) = self.hooks.current_component_id() {
      if let Some(descendants) = self.descendants.get_mut(&parent_component_id) {
        descendants.insert(component.id);
      }
    }

    let old_descendants = self.descendants.remove(&component.id);
    self.descendants.insert(component.id, HashSet::new());

    // `component.render()` is wrapped in an `AnyElement::new` in order to ensure that every
    // component returns a unique container for its elements. This is so on rerenders, when
    // calling `AnyElement::swap`, we know we are affecting only a single component.
    self.hooks.push_cursor(component.id);
    let element = AnyElement::new(component.render(self));
    self.hooks.pop_cursor();

    if let Some(old_descendants) = old_descendants {
      let new_descendants = self.descendants.get(&component.id).expect("DESCENDANTS::get").clone();

      for unmounted_component_id in old_descendants.difference(&new_descendants).cloned() {
        self.unmount(unmounted_component_id);
      }
    }

    element
  }

  /// Unmounts the component.
  fn unmount(&mut self, component_id: ComponentID) {
    self.components.remove(&component_id);
    self.elements.remove(&component_id);

    let descendants = self.descendants.remove(&component_id);

    if let Some(descendants) = descendants {
      for descendant_component_id in descendants {
        self.unmount(descendant_component_id);
      }
    }

    self.hooks.unmount(component_id);
  }
}
