use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Cursor stack is empty")]
  NoCursor,

  #[error("Hooks::get before sealed")]
  NotSealed,

  #[error("Hooks::push after sealed")]
  Sealed,

  #[error("Hooks::seal after sealed")]
  AlreadySealed,

  #[error("Hooks::get on invalid index: {0}")]
  InvalidIdx(usize),

  #[error("Invalid type: {0}")]
  InvalidType(&'static str),
}
