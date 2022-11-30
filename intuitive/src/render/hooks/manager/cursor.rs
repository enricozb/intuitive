#[allow(unused)]
use crate::render::render;
use crate::render::ComponentID;

/// A cursor for reading memoized hook values during [`fn@render`] calls.
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
  pub component_id: ComponentID,
  pub idx: usize,
  pub mode: Mode,
}

impl Cursor {
  /// Creates a new [`Cursor`].
  pub fn new(component_id: ComponentID, mode: Mode) -> Self {
    Self {
      component_id,
      idx: 0,
      mode,
    }
  }

  /// Increments the cursor's index;
  pub fn increment(&mut self) {
    self.idx += 1;
  }
}

/// A cursor's mode.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
  Writing,
  Reading,
}
