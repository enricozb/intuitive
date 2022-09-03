use std::sync::Arc;

use parking_lot::Mutex;

use crate::event;

#[derive(Default)]
pub struct State<T> {
  inner: Arc<Mutex<T>>,
}

impl<T> State<T> {
  pub fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  pub fn set(&self, new: T) {
    let mut inner = self.inner.lock();
    *inner = new;

    event::re_render().expect("re_render");
  }

  pub fn mutate<F>(&self, f: F)
  where
    F: FnOnce(&mut T),
  {
    let mut inner = self.inner.lock();
    f(&mut inner);

    event::re_render().expect("re_render");
  }

  pub fn update<F>(&self, f: F)
  where
    F: FnOnce(&T) -> T,
  {
    let mut inner = self.inner.lock();
    *inner = f(&inner);

    event::re_render().expect("re_render");
  }
}

impl<T: Clone> State<T> {
  pub fn get(&self) -> T {
    self.inner.lock().clone()
  }
}

impl<T> Clone for State<T> {
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}
