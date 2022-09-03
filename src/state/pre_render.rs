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

  pub fn done(&self) {
    self.is_done.lock().set(true);
  }

  pub fn is_done(&self) -> bool {
    self.is_done.lock().get()
  }
}
