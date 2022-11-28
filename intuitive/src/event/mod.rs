mod channel;
use std::thread;

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent};

pub(crate) use self::channel::{read, send};

pub(crate) enum Event {
  Resize,
  Quit,
}

pub(crate) fn start_crossterm_events() {
  thread::spawn(move || loop {
    let event = match crossterm_event::read().expect("read") {
      CrosstermEvent::Resize(..) => Event::Resize,

      _ => continue,
    };

    channel::send(event).expect("send");
  });
}
