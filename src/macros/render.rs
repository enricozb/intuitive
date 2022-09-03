#[macro_export]
macro_rules! render {
  // component with children
  (
    $component:ident($($prop:ident: $value:expr),* $(,)*) { $($children:tt)* }
  ) => {
    $component {
      $($prop: $value.try_into().expect(&format!("try_info failed for argument '{}'", stringify!($prop))),)*

      children: $crate::render! { @children [] $($children)* },

      ..Default::default()
    }.into()
  };

  // component without children
  (
    $component:ident($($prop:ident: $value:expr),* $(,)*)
  ) => {
    $component {
      $($prop: $value.try_into().expect(&format!("try_info failed for argument '{}'", stringify!($prop))),)*

      ..Default::default()
    }.into()
  };

  // push-down accumulator for parsing children
  (
    @children [$($parsed:tt)*]
  ) => {
    [ $($parsed)* ].into()
  };

  (
    @children [$($parsed:tt)*] $component:ident($($prop:ident: $value:expr),* $(,)*) { $($children:tt)* } $($rest:tt)*
  ) => {
    $crate::render! { @children [ $($parsed)* $crate::render! { $component($($prop: $value,)*) { $($children)* } }, ] $($rest)* }
  };

  (
    @children [$($parsed:tt)*] $component:ident($($prop:ident: $value:expr),* ) $($rest:tt)*
  ) => {
    $crate::render! { @children [ $($parsed)* $crate::render! { $component($($prop: $value,)*) }, ] $($rest)* }
  };
}
