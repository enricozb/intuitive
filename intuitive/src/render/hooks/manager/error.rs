use thiserror::Error;

use crate::render::ComponentID;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Cursor stack is empty")]
  NoCursor,

  #[error("No Memos for component: {0:?}")]
  NoMemo(ComponentID),
}