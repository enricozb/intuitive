//! Primitives for handling and sending events.

mod channel;
mod handler;

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub use self::{
  channel::{quit, re_render, read, send, start_crossterm_events},
  handler::Handler as KeyHandler,
};

pub enum Event {
  Key(KeyEvent),
  Render,
  Quit,
}
