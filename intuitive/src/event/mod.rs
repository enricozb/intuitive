//! Primitives for handling and sending events.

mod channel;
mod handler;

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub use self::channel::{quit, re_render};
pub(crate) use self::channel::{read, start_crossterm_events};
use self::handler::Handler;
use crate::terminal::Rect;

pub(crate) enum Event {
  Mouse(MouseEvent),
  Key(KeyEvent),
  Render,
  Quit,
}

pub type KeyHandler = Handler<KeyEvent>;
pub type MouseHandler = Handler<MouseEvent>;

pub fn is_within(event: &MouseEvent, rect: Rect) -> bool {
  let (x, y) = (event.column, event.row);

  let x_within = rect.x <= x && x <= rect.x + rect.width;
  let y_within = rect.y <= y && y <= rect.y + rect.height;

  x_within && y_within
}
