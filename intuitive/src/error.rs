//! Intuitive error types.

use std::{io::Error as IoError, sync::mpsc::RecvError};

use thiserror::Error;

/// Intuitive result.
pub type Result<T> = std::result::Result<T, Error>;

/// Intuitive errors.
#[derive(Error, Debug)]
pub enum Error {
  #[error("Narrower region exceeds bounds")]
  RegionOutOfBounds,

  #[error("io: {0}")]
  Io(#[from] IoError),

  #[error("send: {0}")]
  Send(String),

  #[error("recv: {0}")]
  Recv(#[from] RecvError),
}
