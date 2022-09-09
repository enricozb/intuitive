use std::{ops::Deref, sync::Arc};

use super::{Component, Empty};

/// An opaque type holding a struct that implements [`Component`].
///
/// `Any` is rarely used directly, even when implementing [`Component`].
/// It is typically indirectly used when a component receives children,
/// through [`Children<N>`].
///
/// An example of a component that requires `Any` is [`Modal`]. This is
/// because it provides a function, [`Funcs::show`], that receives a component
/// and presents it.
///
/// [`Children<N>`]: children/struct.Children.html
/// [`Component`]: trait.Component.html
/// [`Funcs::show`]: modal/struct.Funcs.html#method.show
/// [`Modal`]: modal/struct.Modal.html
#[derive(Clone)]
pub struct Any(Arc<dyn Component + 'static + Send + Sync>);

impl Any {
  fn new<C: Component + 'static + Send + Sync>(component: C) -> Self {
    Any(Arc::new(component))
  }
}

impl Default for Any {
  fn default() -> Self {
    Empty.into()
  }
}

impl Deref for Any {
  type Target = Arc<dyn Component + Send + Sync>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<C: Component + 'static + Send + Sync> From<C> for Any {
  fn from(component: C) -> Self {
    Self::new(component)
  }
}
