use crate::{components::Any as AnyComponent, element::Any as AnyElement, error::Result, render::ComponentID};

/// The rendering context.
///
/// This keeps track of all inter-render data in order to support:
/// - hooks
/// - unmounting components
/// - re-renders
pub struct Context {}

impl Context {
  /// Creates a new [`Context`].
  pub fn new() -> Self {
    Self {}
  }

  /// Renders a component.
  pub fn render(&mut self, component: AnyComponent) -> AnyElement {}

  /// Re-renders a component.
  pub(crate) fn rerender(&mut self, component_id: ComponentID) -> Result<()> {
    Ok(())
  }
}
