use thiserror::Error;

use crate::render::hooks::error::Error as HookError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("HookError: {0}")]
  HookError(HookError),
}
