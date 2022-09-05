#[macro_export]
macro_rules! on_key {
  (
    [ $($capture:ident),* ] $($children:tt)*
  ) => {
    {
      $(
        let $capture = $capture.clone();
      )*

      move |event| {
        use $crate::event::KeyCode::*;

        match event {
          $($children)*

          _ => (),
        }
      }
    }
  };
}
