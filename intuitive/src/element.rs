//! Types describing rendered components.

use std::{ops::Deref, sync::Arc};

use crate::{
  components::{Component, Empty},
  event::{KeyEvent, MouseEvent},
  terminal::{Frame, Rect},
};

/// An opaque type holding a struct that implements [`Element`].
///
/// [`Element`]: trait.Element.html
#[derive(Clone)]
pub struct Any(Arc<dyn Element + Send + Sync>);

impl Any {
  pub fn new<C: Element + 'static + Send + Sync>(element: C) -> Self {
    Self(Arc::new(element))
  }
}

impl Default for Any {
  fn default() -> Self {
    Empty {}.into()
  }
}

impl Deref for Any {
  type Target = Arc<dyn Element + Send + Sync>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<C: Component + 'static> From<C> for Any {
  fn from(component: C) -> Self {
    component.render()
  }
}

/// A rendered component.
///
/// Once a [`Component`] is rendered, it now can be drawn (through [`draw`]) or it
/// can handle a key event (through [`on_key`]).
///
/// # Drawing
/// Intuitive internally uses [tui] in order to draw to the terminal. The [`Rect`]
/// and [`Frame`] structures are re-exports from [tui].
///
/// # Handling Keys
/// Typically, structures that implement `Element` do not have any [`State`].
/// Usually, an `Element` will contain an `on_key` field which has captured any
/// state that could be mutated, and then the `Element` will delegate key events
/// to its `on_key` field. See the [`Section` source] for an example of this.
///
/// [`Component`]: ../components/trait.Component.html
/// [`draw`]: #method.draw
/// [`Frame`]: https://docs.rs/tui/latest/tui/terminal/struct.Frame.html
/// [`on_key`]: #method.on_key
/// [`Rect`]: https://docs.rs/tui/latest/tui/layout/struct.Rect.html
/// [`Section` source]: ../../src/intuitive/components/section.rs.html
/// [`State`]: ../state/struct.State.html
/// [tui]: https://docs.rs/tui/latest/tui/
pub trait Element {
  fn draw(&self, _rect: Rect, _frame: &mut Frame) {}
  fn on_key(&self, _event: KeyEvent) {}
  fn on_mouse(&self, _rect: Rect, _event: MouseEvent) {}
}
