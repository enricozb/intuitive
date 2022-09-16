//! Primitives for handling and sending events.

mod channel;
pub mod handler;

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

/// A handler for [`KeyEvent`]s.
///
/// # Creating a `KeyHandler`
///
/// A `KeyHandler` is used to manipulate closures or functions that take in a
/// [`KeyEvent`] as a parameter. It implements `From<Fn(KeyEvent) + 'static + Send + Sync>`,
/// which allows for code like this:
/// ```rust
/// # use intuitive::{component, components::Text, render, on_key};
/// #
/// #[component(Root)]
/// fn render() {
///   let on_key = on_key! {};
///
///   render! {
///     Text(text: "Hi There", on_key)
///   }
/// }
/// ```
///
/// `KeyHandler`s are often constructed using the [`on_key!`] macro.
///
/// # Using a `KeyHandler`
///
/// A [`KeyEvent`] can be handled by a `KeyHandler` through the [`KeyHandler::handle`]
/// method. `KeyHandler` implements [`Default`], and the default handler ignores the
/// `KeyEvent`. Typically, components want to take some default action when implementing
/// `on_key`, but allow the user of this component to override this handler. This can
/// be done using the [`KeyHandler::handle_or`] method:
/// ```rust
/// # use intuitive::{element::Element, event::{KeyHandler, KeyEvent}};
/// #
/// struct Frozen {
///   on_key: KeyHandler,
/// }
///
/// impl Element for Frozen {
///   fn on_key(&self, event: KeyEvent) {
///     self.on_key.handle_or(event, |event| { /* default functionality here */ })
///   }
/// }
/// ```
/// Here, `Frozen::on_key` calls the handler that was provided if one was. If no
/// non-default `KeyHandler` was provided, then the closure passed to
/// [`KeyHandler::handle_or`] is executed.
///
/// [`KeyEvent`]: struct.KeyEvent.html
/// [`on_key!`]: ../macro.on_key.html
/// [`KeyHandler::handle`]: #method.handle
/// [`KeyHandler::handle_or`]: #method.handle_or
/// [`State`]: ../state/struct.State.html
/// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
pub type KeyHandler = Handler<KeyEvent>;

/// A handler for [`KeyEvent`]s.
///
/// [`MouseEvent`]: struct.MouseEvent.html
pub type MouseHandler = Handler<MouseEvent>;

pub fn is_within(event: &MouseEvent, rect: Rect) -> bool {
  let (x, y) = (event.column, event.row);

  let x_within = rect.x <= x && x <= rect.x + rect.width;
  let y_within = rect.y <= y && y <= rect.y + rect.height;

  x_within && y_within
}
