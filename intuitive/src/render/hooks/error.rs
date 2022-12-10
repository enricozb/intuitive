use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Cursor stack is empty")]
  NoCursor,

  #[error("invalid index: {0}")]
  InvalidIndex(usize),

  #[error("invalid type: {0}")]
  InvalidType(&'static str),
}
