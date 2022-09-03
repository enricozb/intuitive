use std::any::Any;

use super::State;
use crate::error::{Error, Result};

pub struct Manager {
  states: Vec<Box<dyn Any + Send + Sync>>,
  idx: usize,
  filling: bool,
}

impl Manager {
  pub fn new() -> Self {
    Self {
      states: Vec::new(),
      idx: 0,
      filling: true,
    }
  }

  pub fn next<T, F>(&mut self, initializer: F) -> Result<State<T>>
  where
    T: 'static + Send,
    F: FnOnce() -> T,
  {
    if self.filling {
      let state = State::new(initializer());
      self.states.push(Box::new(state.clone()));

      Ok(state)
    } else {
      let state = self.states.get(self.idx).ok_or(Error::Manager("invalid index"))?;

      self.idx += 1;

      Ok(state.downcast_ref::<State<T>>().ok_or(Error::Manager("invalid type"))?.clone())
    }
  }

  pub fn reset(&mut self) -> Result<()> {
    if self.filling {
      self.filling = false;
    } else if self.idx != self.states.len() {
      return Err(Error::Manager("insufficient calls"));
    }

    self.idx = 0;

    Ok(())
  }
}
