//! Primitives for handling state.

mod hook;
mod manager;

use std::sync::Arc;

pub use hook::{render_done, use_state};
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

  pub fn inspect<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&self.inner.lock())
  }

  pub fn mutate<F, R>(&self, f: F)
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut inner = self.inner.lock();
    drop(f(&mut inner));

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