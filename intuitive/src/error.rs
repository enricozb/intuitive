//! The crate's `Error` type.

use std::{io, sync::mpsc::RecvError};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("io: {0}")]
  Io(#[from] io::Error),

  #[error("recv: {0}")]
  Recv(#[from] RecvError),

  #[error("send: {0}")]
  Send(String),

  #[error("manager: {0}")]
  Manager(&'static str),

  #[error("use_state calls must be in the same order: {0}")]
  UseState(String),
}
