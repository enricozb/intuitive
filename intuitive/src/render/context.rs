//! The [`Context`] type.

use super::providers::{Components, Descendants, Elements, Hooks};
use crate::{components::Any as AnyComponent, element::Any as AnyElement, render::ComponentID, utils::provider::Provider};

/// The rendering context.
///
/// This keeps track of all inter-render data in order to support:
/// - hooks
/// - unmounting components
/// - re-renders
#[derive(Default)]
pub struct Context {
  hooks: Hooks,
  descendants: Descendants,
  elements: Elements,
  components: Components,
}

impl Context {
  /// Renders a component.
  pub fn render(&mut self, component: &AnyComponent) -> AnyElement {
    self.render_impl(component, false)
  }

  /// Returns a mutable reference to the inner [`Hooks`].
  pub fn hooks(&mut self) -> &mut Hooks {
    &mut self.hooks
  }

  /// Re-renders a component.
  pub(crate) fn rerender(&mut self, component_id: ComponentID) {
    if let Some(component) = self.components.get(&component_id).cloned() {
      self.render_impl(&component, true);
    }
  }

  fn render_impl(&mut self, component: &AnyComponent, is_rerender: bool) -> AnyElement {
    // https://github.com/rust-lang/rust/issues/86935#issuecomment-1146670057
    type Type<T> = T;

    self.components.enter(component);
    self.elements.enter(&());
    self.hooks.enter(&component.id);
    let old_descendants = self.descendants.enter(&component.id);

    let element = component.render(self);

    let unmounted_component_ids = self.descendants.exit(&old_descendants);
    self.hooks.exit(&unmounted_component_ids).expect("hooks::exit");
    self.components.exit(&unmounted_component_ids);
    self.elements.exit(&Type::<<Elements as Provider>::Exit> {
      component_id: component.id,
      element: element.clone(),
      is_rerender,
    });

    element
  }
}
