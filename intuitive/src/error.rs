use thiserror::Error;

use crate::render::{hooks::error::Error as HookError, ComponentID};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Hook error (perhaps hooks are called in non-deterministic order?): {0}")]
  HookError(HookError),

  #[error("No component found for the given component id: {0:?}")]
  NoComponent(ComponentID),

  #[error("No element found for the given component id: {0:?}")]
  NoElement(ComponentID),
}
