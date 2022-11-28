use std::sync::{
  mpsc::{self, Receiver, Sender},
  Arc,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use super::Event;
use crate::error::{Error, Result};

lazy_static! {
  /// The global [`Channel`].
  static ref CHANNEL: Channel = Channel::new();
}

/// Reads from the global [`Channel`].
pub(crate) fn read() -> Result<Event> {
  CHANNEL.read()
}

/// Sends to the global [`Channel`].
pub(crate) fn send(event: Event) -> Result<()> {
  CHANNEL.send(event)
}

/// A channel used to send/read [`Event`]s to/from.
struct Channel {
  sender: Arc<Mutex<Sender<Event>>>,
  reader: Arc<Mutex<Receiver<Event>>>,
}

impl Channel {
  /// Creates a new [`Channel`].
  pub fn new() -> Self {
    let (sender, reader) = mpsc::channel();

    Self {
      sender: Arc::new(Mutex::new(sender)),
      reader: Arc::new(Mutex::new(reader)),
    }
  }

  /// Reads an [`Event`], blocking if none is present.
  pub fn read(&self) -> Result<Event> {
    Ok(self.reader.lock().recv()?)
  }

  /// Sends an [`Event`].
  pub fn send(&self, event: Event) -> Result<()> {
    self.sender.lock().send(event).map_err(|err| Error::Send(err.to_string()))
  }
}
