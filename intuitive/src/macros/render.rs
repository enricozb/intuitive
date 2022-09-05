/// Macro for rendering components.
///
/// # Usage
/// This macro is meant to be used when returning from `Component::render`,
/// and uses a [SwiftUI](https://developer.apple.com/xcode/swiftui/)-like syntax.
///
/// For example:
/// ```
/// render! {
///   VStack() {
///     Section(title: "Top Section") {
///       Text(text: "Hello")
///     }
///
///     Section(title: "Bottom Section") {
///       Text(text: "World")
///     }
///   }
/// }
/// ```
/// is rendering a `VStack` (with default parameters), and two children. The
/// child components are `Section`s, each with their own `Text` child components.
///
/// # Parameters
/// Parameters passed to components are passed like function arguments, but they
/// must have the parameter name attached to them. They can also be passed in any order.
/// Components are required to implement `Default`, so all parameters are optional.
///
/// # Children
/// Children to a component come after the component surrounded by braces (`{ ... }`).
/// Like parameters, children are optional, but are only valid for components that
/// accept them (for example `Text` accepts no children, but `Section` does).
///
/// Children are passed as arrays (`[AnyComponent; N]`), so components specify exactly
/// how many children they take in. Some components, like `VStack` and `HStack` take
/// in a variable number of children, while some, like `Section`, only accept a single
/// child component.
#[macro_export]
macro_rules! render {
  // component with children
  ($component:ident($($prop:ident: $value:expr),* $(,)*) { $($children:tt)* }) => {
    $component {
      $($prop: $value.try_into().expect(&format!("try_info failed for argument '{}'", stringify!($prop))),)*

      children: $crate::render! { @children [] $($children)* },

      ..Default::default()
    }.into()
  };

  // component without children
  ($component:ident($($prop:ident: $value:expr),* $(,)*)) => {
    $component {
      $($prop: $value.try_into().expect(&format!("try_info failed for argument '{}'", stringify!($prop))),)*

      ..Default::default()
    }.into()
  };

  // push-down accumulator for parsing children
  (@children [$($parsed:tt)*]) => {
    [ $($parsed)* ].into()
  };

  (@children [$($parsed:tt)*] $component:ident($($prop:ident: $value:expr),* $(,)*) { $($children:tt)* } $($rest:tt)*) => {
    $crate::render! { @children [ $($parsed)* $crate::render! { $component($($prop: $value,)*) { $($children)* } }, ] $($rest)* }
  };

  (@children [$($parsed:tt)*] $component:ident($($prop:ident: $value:expr),* ) $($rest:tt)*) => {
    $crate::render! { @children [ $($parsed)* $crate::render! { $component($($prop: $value,)*) }, ] $($rest)* }
  };
}
