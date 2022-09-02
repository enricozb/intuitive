mod channel;

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub use self::channel::{quit, read, send, start_crossterm_events};

pub enum Event {
  Key(KeyEvent),
  Render,
  Quit,
}
