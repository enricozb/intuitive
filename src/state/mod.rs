mod pre_render;

use std::{any::Any, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use pre_render::PRE_RENDER;

use crate::event;

lazy_static! {
  static ref STATES: Mutex<Vec<Box<dyn Any + Send + Sync>>> = Mutex::new(Vec::new());
  static ref STATES_IDX: Mutex<usize> = Mutex::new(0);
}

#[derive(Default)]
pub struct State<T> {
  inner: Arc<Mutex<T>>,
}

impl<T> State<T> {
  fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  pub fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
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

pub fn pre_render_done() {
  PRE_RENDER.done();
}

pub fn use_state<T, F>(f: F) -> State<T>
where
  T: 'static + Send,
  F: FnOnce() -> T,
{
  let mut states = STATES.lock();

  if PRE_RENDER.is_done() {
    let mut states_idx = STATES_IDX.lock();

    let state = states.get(*states_idx).expect("states::get");

    *states_idx = (*states_idx + 1) % states.len();

    state.downcast_ref::<State<T>>().expect("downcast").clone()
  } else {
    let state = State::new(f());

    states.push(Box::new(state.clone()));

    state
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() {
    // reset prerender and states
    PRE_RENDER.reset();
    STATES.lock().clear();
    *STATES_IDX.lock() = 0;

    let _ = use_state(|| 1);
    let _ = use_state(|| 2);

    PRE_RENDER.done();
  }

  #[test]
  fn use_state_no_panic() {
    setup();

    let _ = use_state(|| 1);
    let _ = use_state(|| 2);
  }

  #[test]
  fn use_state_get() {
    setup();

    let state_1 = use_state(|| 1);
    let state_2 = use_state(|| 2);

    assert_eq!(state_1.get(), 1);
    assert_eq!(state_2.get(), 2);
  }

  #[test]
  fn use_state_set_get() {
    setup();

    let state_1 = use_state(|| 1);
    let state_2 = use_state(|| 2);

    state_1.set(3);
    state_2.set(4);

    assert_eq!(state_1.get(), 3);
    assert_eq!(state_2.get(), 4);
  }

  #[test]
  #[should_panic]
  fn use_state_wrong_type() {
    setup();

    let _ = use_state(|| ());
  }
}
