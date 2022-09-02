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
use crate::error::Result;

lazy_static! {
  static ref CHANNEL: Channel = Channel::new();
}

pub fn read() -> Result<Event> {
  Ok(CHANNEL.receiver.lock().recv()?)
}

pub fn send(event: Event) -> Result<()> {
  Ok(CHANNEL.sender.lock().send(event)?)
}

pub fn quit() -> Result<()> {
  Ok(CHANNEL.sender.lock().send(Event::Quit)?)
}

pub fn start_crossterm_events() {
  let sender = CHANNEL.sender();

  thread::spawn(move || loop {
    let event = match crossterm_event::read().expect("read") {
      CrosstermEvent::Key(event) => Event::Key(event),
      CrosstermEvent::Resize(..) => Event::Render,

      _ => continue,
    };

    sender.send(event).expect("send");
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

  pub fn sender(&self) -> Sender<Event> {
    self.sender.lock().clone()
  }
}
