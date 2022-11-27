use crate::render::ComponentID;

pub struct Cursor {
  pub id: ComponentID,
  pub idx: usize,
  pub mode: Mode,
}

impl Cursor {
  pub fn new(id: ComponentID, mode: Mode) -> Self {
    Self { id, idx: 0, mode }
  }
}

pub enum Mode {
  Writing,
  Reading,
}
