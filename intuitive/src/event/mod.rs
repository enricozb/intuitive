mod channel;
use std::thread;

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent};

pub(crate) use self::channel::read;
use crate::{error::Result, render::ComponentID};

///
pub(crate) enum Event {
  Rerender(ComponentID),
  Resize,
  Quit,
}

/// Queues a [`Event::Rerender`] for the provided `component_id`.
///
/// # Errors
///
/// Will return an `Err` if the event cannot be enqueued.
#[allow(rustdoc::private_intra_doc_links)]
pub fn rerender(component_id: ComponentID) -> Result<()> {
  channel::send(Event::Rerender(component_id))
}

/// Queues a [`Event::Resize`] event.
///
/// # Errors
///
/// Will return an `Err` if the event cannot be enqueued.
#[allow(rustdoc::private_intra_doc_links)]
pub fn resize() -> Result<()> {
  channel::send(Event::Resize)
}

/// Queues a [`Event::Quit`] event.
///
/// # Errors
///
/// Will return an `Err` if the event cannot be enqueued.
#[allow(rustdoc::private_intra_doc_links)]
pub fn quit() -> Result<()> {
  channel::send(Event::Quit)
}

/// Starts the event loop that enqueues incoming [`crossterm::event::Event`]s as [`Event`]s.
pub(crate) fn start_crossterm_events() {
  thread::spawn(move || loop {
    let event = match crossterm_event::read().expect("read") {
      CrosstermEvent::Resize(..) => Event::Resize,

      _ => continue,
    };

    channel::send(event).expect("send");
  });
}
