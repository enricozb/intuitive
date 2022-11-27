pub mod hooks;

use std::collections::HashMap;

use lazy_static::lazy_static;
use parking_lot::Mutex;

use self::hooks::manager;
use crate::{
  components::Any as AnyComponent,
  element::Any as AnyElement,
  error::{Error, Result},
};
#[allow(unused)]
use crate::{components::Component, render};

lazy_static! {
  /// A map from [`ComponentID`] to the [`Component`] that was rendered.
  static ref COMPONENTS: Mutex<HashMap<ComponentID, AnyComponent>> = Mutex::new(HashMap::new());
  /// A map from [`ComponentID`] to the [`Element`] that the result of a [`Component::render`] call.
  static ref ELEMENTS: Mutex<HashMap<ComponentID, AnyElement>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A unique identifier associated with each instance of a [`Component`] used *across all* [`render!`] macros.
/// *This should rarely if ever be used manually*. These structs are automatically created when using the
/// [`render!`] macro. These fields are public because they must be referenced within the code generated by the
/// [`render!`] macro.
pub struct ComponentID {
  /// The name of the component.
  pub name: &'static str,
  /// The `key` parameter, if any, that was provided.
  pub key: Option<usize>,
  /// The file the component was used in.
  pub file: &'static str,
  /// An integer unique to each instance of a [`Component`] used across all [`render!`] macros.
  pub uid: usize,
}

/// Renders a component.
///
/// A [`ComponentID`] is required because it is used to track which hooks are used within the rendering of the
/// specific instance of a component.
pub fn render<C: Component + 'static + Send>(id: ComponentID, component: C) -> AnyElement {
  let component = AnyComponent::new(component);

  manager::with(id, || {
    let element = component.render();

    COMPONENTS.lock().insert(id, component);
    ELEMENTS.lock().insert(id, element.clone());

    element
  })
}

/// Re-renders an already rendered component, specified by its [`ComponentID`].
///
/// # Errors
///
/// Will return an error if a component has not yet been rendered with the provided [`ComponentID`].
pub fn rerender(id: ComponentID) -> Result<()> {
  manager::with(id, || -> Result<()> {
    let components = COMPONENTS.lock();
    let elements = ELEMENTS.lock();

    let component = components.get(&id).ok_or(Error::NoComponent(id))?;
    let element = elements.get(&id).ok_or(Error::NoElement(id))?;

    element.swap(&component.render());

    Ok(())
  })
}
