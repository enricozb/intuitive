//! Primitives for handling and sending events.

mod channel;
mod handler;

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) use self::channel::{read, start_crossterm_events};
pub use self::{
  channel::{quit, re_render},
  handler::Handler as KeyHandler,
};

pub(crate) enum Event {
  Key(KeyEvent),
  Render,
  Quit,
}
