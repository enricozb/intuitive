use std::{
  sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
  },
  thread,
};

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent};
use lazy_static::lazy_static;
use parking_lot::Mutex;

use super::Event;
use crate::error::{Error, Result};

lazy_static! {
  static ref CHANNEL: Channel = Channel::new();
}

pub(crate) fn read() -> Result<Event> {
  CHANNEL.recv()
}

fn send(event: Event) -> Result<()> {
  CHANNEL.send(event)
}

/// Triggers a re-render.
pub fn re_render() -> Result<()> {
  send(Event::Render)
}

/// Quits the application.
///
/// This is often used in [`KeyHandler`]s like so:
/// ```rust
/// # use intuitive::on_key;
/// #
/// let on_key = on_key! {
///   KeyEvent { code: Char('q'), .. } => event::quit(),
/// };
/// ```
///
/// [`KeyHandler`]: struct.KeyHandler.html
pub fn quit() {
  send(Event::Quit).expect("quit")
}

pub fn start_crossterm_events() {
  thread::spawn(move || loop {
    let event = match crossterm_event::read().expect("read") {
      CrosstermEvent::Key(event) => Event::Key(event),
      CrosstermEvent::Resize(..) => Event::Render,

      _ => continue,
    };

    send(event).expect("send");
  });
}

struct Channel {
  sender: Arc<Mutex<Sender<Event>>>,
  receiver: Arc<Mutex<Receiver<Event>>>,
}

impl Channel {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();

    Self {
      sender: Arc::new(Mutex::new(sender)),
      receiver: Arc::new(Mutex::new(receiver)),
    }
  }

  pub fn recv(&self) -> Result<Event> {
    Ok(self.receiver.lock().recv()?)
  }

  pub fn send(&self, event: Event) -> Result<()> {
    self.sender.lock().send(event).map_err(|err| Error::Send(err.to_string()))
  }
}
