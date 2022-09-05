/// Helper macro for creating key handlers.
///
/// # Details
/// This macro is used to simplify a common pattern of key handlers where:
/// - `event`, `event::KeyEvent` and `event::KeyCode::*` are brought into scope
/// - `State`s need to be cloned before being moved into the key handler
/// - The event is immediately `match`ed
///
/// In addition to the above, this macro also:
/// - implicitly introduces the `|event|` closure parameter
/// - adds the catch-all `_ => ()` case to the `match` expression
///
/// # Usage
/// An example usage looks like the following:
/// ```rust
/// let text = use_state(String::new);
///
/// let on_key = on_key! { [text]
///   KeyEvent { code: Char(c), .. } => text.update(|text| text.push(c)),
///   KeyEvent { code: Char(c), .. } => text.update(|text| text.pop()),
/// };
/// ```
/// and expands to the following:
/// ```rust
/// let text = use_state(String::new);
///
/// let on_key = {
///   let text = text.clone();
///
///   move |event| {
///     use intuitive::event::{self, KeyEvent, KeyCode::*};
///
///     match event {
///       KeyEvent { code: Char(c), .. } => text.update(|text| text.push(c)),
///       KeyEvent { code: Char(c), .. } => text.update(|text| text.pop()),
///       _ => (),
///     };
///   };
/// };
/// ```
/// Notice that a trailing comma is required in this macro, as `_ => ()` is
/// always added as the last arm of the match expression.
#[macro_export]
macro_rules! on_key {
  ([ $($capture:ident),* ] $($children:tt)*) => {
    {
      $(
        let $capture = $capture.clone();
      )*

      move |event| {
        use $crate::event::{self, KeyCode::*, KeyEvent};

        match event {
          $($children)*

          _ => (),
        }
      }
    }
  };

  ($($children:tt)*) => {
    move |event| {
      use $crate::event::{self, KeyCode::*, KeyEvent};

      match event {
        $($children)*

        _ => (),
      };
    }
  };
}
