use std::collections::HashMap;

use super::providers::{Descendants, Elements, Hooks};
use crate::{components::Any as AnyComponent, element::Any as AnyElement, error::Result, render::ComponentID, utils::provider::Provider};

/// The rendering context.
///
/// This keeps track of all inter-render data in order to support:
/// - hooks
/// - unmounting components
/// - re-renders
pub struct Context {
  pub hooks: Hooks,
  descendants: Descendants,
  elements: Elements,

  /// Components that have been rendered.
  components: HashMap<ComponentID, AnyComponent>,
}

impl Context {
  /// Creates a new [`Context`].
  pub fn new() -> Self {
    Self {
      hooks: Hooks::new(),
      descendants: Descendants::new(),
      elements: Elements::new(),

      components: HashMap::new(),
    }
  }

  /// Renders a component.
  pub fn render(&mut self, component: AnyComponent) -> AnyElement {
    self.components.insert(component.id, component.clone());

    self.render_impl(component, false)
  }

  /// Re-renders a component.
  pub(crate) fn rerender(&mut self, component_id: ComponentID) -> Result<()> {
    if let Some(component) = self.components.get(&component_id) {
      self.render_impl(component.clone(), true);
    }

    Ok(())
  }

  fn render_impl(&mut self, component: AnyComponent, is_rerender: bool) -> AnyElement {
    // https://github.com/rust-lang/rust/issues/86935#issuecomment-1146670057
    type Type<T> = T;

    let () = self.elements.enter(());
    let () = self.hooks.enter(component.id);
    let old_descendants = self.descendants.enter(component.id);

    let element = component.render(self);

    let unmounted_component_ids = self.descendants.exit(old_descendants, ());

    for component_id in &unmounted_component_ids {
      self.components.remove(&component_id);
    }

    let () = self.hooks.exit((), unmounted_component_ids).expect("hooks::exit");
    let () = self.elements.exit(
      (),
      Type::<<Elements as Provider>::Exit> {
        component_id: component.id,
        element: element.clone(),
        is_rerender,
      },
    );

    element
  }
}
