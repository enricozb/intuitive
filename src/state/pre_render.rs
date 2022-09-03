use std::cell::Cell;

use parking_lot::Mutex;

pub static PRE_RENDER: PreRender = PreRender::new();

pub struct PreRender {
  is_done: Mutex<Cell<bool>>,
}

impl PreRender {
  const fn new() -> Self {
    Self {
      is_done: Mutex::new(Cell::new(false)),
    }
  }

  pub fn set_done(&self) {
    self.is_done.lock().set(true);
  }

  #[cfg(test)]
  pub fn reset(&self) {
    self.is_done.lock().set(false);
  }

  pub fn is_done(&self) -> bool {
    self.is_done.lock().get()
  }
}
