//! Primitives for handling and sending events.

mod channel;
pub mod handler;

pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub use self::channel::{quit, re_render};
pub(crate) use self::channel::{read, start_crossterm_events};
use self::handler::Handler;
#[cfg(doc)]
use self::handler::Propagate;
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
/// [`KeyEvent`] as a parameter, and return a [`Propagate`]. `KeyHandler`s are often
/// created using the [`on_key!`] macro. For example,
/// ```rust
/// # use intuitive::{component, components::Text, render, on_key, state::use_state};
/// #
/// #[component(Root)]
/// fn render() {
///   let text = use_state(|| String::new());
///
///   let on_key = on_key! { [text]
///     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
///   };
///
///   render! {
///     Text(text: format!("Hi There {}", text.get()), on_key)
///   }
/// }
/// ```
///
/// # Using a `KeyHandler`
///
/// A [`KeyEvent`] can be handled by a `KeyHandler` through the [`Handler::handle`]
/// method. `KeyHandler` implements [`Default`], and the default handler ignores the
/// `KeyEvent`, and always returns [`Propagate`]`::Next`.
///
/// Typically, components want to take some default action when implementing
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
/// `KeyHandler` was provided, then `self.on_key` is the default handler,
/// which always returns [`Propagate`]`::Next`. This causes the closure above to
/// be executed.
///
/// # Propagation
///
/// A user of a component can control when the default key handler is run by
/// returning one of [`Propagate`]`::{Next, Stop}`. For example, to create an
/// input box that receives input keys, but quits on the escape key:
/// ```rust
/// # use intuitive::{component, components::experimental::input::Input, render, on_key, state::use_state};
/// #
/// #[component(Root)]
/// fn render() {
///   let text = use_state(|| String::new);
///
///   let on_key = on_key! { [text]
///     KeyEvent { code: Esc, .. } => event::quit(),
///
///     _ => return Propagate::Next,
///   };
///
///   render! {
///     Input(on_key)
///   }
/// }
/// ```
/// This will cause all key events other than `Esc` to be handled by `Input`'s
/// default key handler.
///
/// [`on_key!`]: ../macro.on_key.html
/// [`Handler::handle_or`]: handler/struct.Handler.html#method.handle_or
/// [`State`]: ../state/struct.State.html
/// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
pub type KeyHandler = Handler<KeyEvent>;

/// A handler for [`MouseEvent`]s.
pub type MouseHandler = Handler<MouseEvent>;

/// Check if a mouse event is within a [`Rect`].
pub fn is_within(event: &MouseEvent, rect: Rect) -> bool {
  let (x, y) = (event.column, event.row);

  let x_within = rect.x <= x && x <= rect.x + rect.width;
  let y_within = rect.y <= y && y <= rect.y + rect.height;

  x_within && y_within
}
