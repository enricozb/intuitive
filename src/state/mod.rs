mod pre_render;

use std::{any::Any, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use pre_render::PRE_RENDER;

lazy_static! {
  static ref STATES: Mutex<Vec<Box<dyn Any + Send + Sync>>> = Mutex::new(Vec::new());
  static ref STATES_IDX: Mutex<usize> = Mutex::new(0);
}

pub struct State<T> {
  inner: Arc<Mutex<T>>,
}

impl<T> Clone for State<T> {
  fn clone(&self) -> Self {
    Self { inner: self.inner.clone() }
  }
}

impl<T> State<T> {
  fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }
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

  #[test]
  fn use_state_no_panic() {
    let _ = use_state(|| 1);
    let _ = use_state(|| "two");

    PRE_RENDER.done();

    let _ = use_state(|| 1);
    let _ = use_state(|| "two");
  }
}
