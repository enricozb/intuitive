use std::{ops::Deref, sync::Arc};

use super::{Component, Empty};

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
