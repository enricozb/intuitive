use std::sync::Arc;

/// A handler for [`KeyEvent`]s.
///
/// # Creating a `KeyHandler`
///
/// A `KeyHandler` is used to manipulate closures or functions that take in a
/// [`KeyEvent`] as a parameter. It implements `From<Fn(KeyEvent) + 'static + Send + Sync>`,
/// which allows for code like this:
/// ```rust
/// # use intuitive::{component, components::Text, render};
/// #
/// #[component(Root)]
/// fn render() {
///   let on_key = |event| {};
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
pub struct Handler<T> {
  handler: Option<Arc<dyn Fn(T) + 'static + Send + Sync>>,
}

impl<T> Default for Handler<T> {
  fn default() -> Self {
    Self { handler: None }
  }
}

impl<T> Clone for Handler<T> {
  fn clone(&self) -> Self {
    Self {
      handler: self.handler.clone(),
    }
  }
}

impl<T> Handler<T> {
  /// Call the inner function on the provided [`KeyEvent`]
  ///
  /// [`KeyEvent`]: struct.KeyEvent.html
  pub fn handle(&self, event: T) {
    self.handle_or(event, |_| {});
  }

  /// If this `KeyHandler` is not the default handler, call the inner function
  /// on the provided [`KeyEvent`]. Otherwise, call the `alternative_handler` on
  /// the provided [`KeyEvent`].
  ///
  /// [`KeyEvent`]: struct.KeyEvent.html
  pub fn handle_or<F>(&self, event: T, alternative_handler: F)
  where
    F: FnOnce(T),
  {
    if let Some(handler) = &self.handler {
      handler(event);
    } else {
      alternative_handler(event);
    }
  }
}

impl<F, T> From<F> for Handler<T>
where
  F: Fn(T) + 'static + Send + Sync,
{
  fn from(f: F) -> Self {
    Self {
      handler: Some(Arc::new(f)),
    }
  }
}

impl<T> From<&Handler<T>> for Handler<T> {
  fn from(handler: &Handler<T>) -> Self {
    handler.clone()
  }
}
