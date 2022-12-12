// A hack so macro uses within intuitive use the correct crate name.
extern crate self as intuitive;

pub mod buffer;
pub mod components;
pub mod element;
pub mod error;
pub mod render;
pub mod style;
pub mod terminal;
pub mod utils;

pub mod event;

/// Helper attribute macro for creating functional components.
///
/// # Usage
/// This macro is used to create functional components, where the name (and type parameters) of the generated component
/// is the item in the attribute. For example,
/// ```rust
/// # use intuitive::{
/// #   component,
/// #   components::{Section, Text},
/// #   element::Any as AnyElement,
/// #   render,
/// # };
/// #
/// #[component(Root)]
/// pub fn render(title: String) -> AnyElement {
///   render! {
///     Section(title) {
///       Text(text: "Hello world!")
///     }
///   }
/// }
/// ```
/// constructs a `Root` component, that can be used in a [`render!`] macro.
///
/// # Parameters
/// If the `render` function contains parameters, these will become parameters to the generated component. These
/// parameters can later be supplied when using the generated component in a [`render!`] macro. The parameters' types
/// **must** implement [`Default`] as the generated component derives [`Default`]. If you need more control over the
/// default values of the parameters, consider implementing the [`Component`] trait instead of using the
/// [`#[component(..)]`](component) attribute macro.
///
/// # Hooks
/// Intuitive supports a construct similar to [React's hooks]. See the [module-level documentation](render::hooks) for
/// details.
///
/// These are functions defined through traits on the [`Hooks`] context provider. A `hooks` variable
/// (of type [`Hooks`]) is implicitly introduced into scope within the function that the
/// [`#[component(..)]`](component) attribute macro is applied to.
///
/// For example, here is a component that shows how many seconds have passed since it was first rendered:
/// ```rust
/// use std::{thread, time::Duration};
///
/// use intuitive::{
///   component,
///   components::{Section, Text},
///   element::Any as AnyElement,
///   render,
///   render::hooks::{UseEffect, UseState},
///   style::Color,
/// };
///
/// #[component(Root)]
/// fn render() -> AnyElement {
///   let seconds = hooks.use_state(|| 0);
///
///   // cloned because it's moved into the `use_effect` hook below
///   let seconds_clone = seconds.clone();
///
///   hooks.use_effect(|| {
///     thread::spawn(move || loop {
///       thread::sleep(Duration::from_secs(1));
///
///       seconds_clone.update(|seconds| seconds + 1).unwrap();
///     });
///   });
///
///   render! {
///     Section(title: "Seconds", border: Color::Red) {
///       Text(text: format!("This program has run for {} seconds", seconds.get()))
///     }
///   }
/// }
///
/// ```
/// Notice the implicit variable `hooks`, and how the [`UseEffect`] and [`UseState`] traits had to be imported.
///
/// # Generics
/// When requiring generics they can be added into the attribute and then used in the parameters. For example,
/// ```rust
/// # use std::fmt::Display;
/// # use intuitive::{
/// #   component,
/// #   components::{Section, Text},
/// #   element::Any as AnyElement,
/// #   error::Result,
/// #   render,
/// #   terminal::Terminal,
/// # };
/// #
/// #[component(Root<T: Display + Default>)]
/// fn render(title: String, t: T) -> AnyElement {
///   render! {
///     Section(title) {
///       Text(text: format!("My value: {t}"))
///     }
///   }
/// }
/// ```
///
/// # Generated Component
/// The generated component is a structure that implements the [`Component`] trait. The structure's fields are exactly
/// the parameters defined in the `render` function passed to [`#[component(..)]`](component).
///
/// # Additional Details + TL;DR
/// 1. [`#[component(..)]`](component) generates a structure that implements [`Component`].
/// 2. The name and type parameters of the generated component are in the item of the attribute macro.
/// 3. The variables `hooks: `[`Hooks`] and `context: `[`Context`] are implicitly introduced into `render`'s scope.
/// 4. The visibility of the generated component will be the same as the visibility of the `render` function the
///    [`#[component(..)]`](component) attribute is applied to.
/// 5. The function name (commonly `render`) is ignored.
/// 6. The return type must be [`AnyElement`].
///
/// [`AnyElement`]: element::Any
/// [`Component`]: components::Component
/// [`Context`]: render::Context
/// [`Hooks`]: render::providers::Hooks
/// [`UseEffect`]: render::hooks::UseEffect
/// [`UseState`]: render::hooks::UseState
/// [React]: https://reactjs.org/
/// [React's hooks]: https://reactjs.org/docs/hooks-intro.html
pub use intuitive_macros::component;
/// Macro for rendering components.
///
/// # Usage
/// This macro is used to render components from within [`Component::render`]. It uses a [SwiftUI-like] syntax, where
/// the rough syntax is as follows:
/// ```text
/// render! { <component> }
///
/// component := <name> ( <parameters> ) { <children> }
/// children := <component> [ <component> ... ]
/// ```
///
/// For example:
/// ```rust
/// # use intuitive::{
/// #   component,
/// #   components::{Section, Text},
/// #   element::Any as AnyElement,
/// #   error::Result,
/// #   render,
/// #   terminal::Terminal,
/// # };
/// #
/// #[component(Root)]
/// fn render() -> AnyElement {
///   render! {
///     Section(title: "Hello, world!") {
///       Text(text: "Here's a basic example of intuitive!")
///     }
///   }
/// }
/// ```
/// is rendering a [`Section`] with a [`Text`] as its child.
///
/// # Parameters
/// Parameters passed to components look like function arguments but are actually much closer to structure
/// initialization. Like struct fields, they can be passed in any order, and they require the field name, unless
/// the parameter and value are the same identifier. Unlike struct fields, you can omit parameters, as any omitted
/// parameters are implicitly passed in with their default values.
///
/// ## Automatic Parameter Conversion
/// When passing parameters to components within a [`render!`] macro invocation, an implicit [`TryInto::try_into`]
/// call is made for each parameter. This means that you can omit most `.into()` calls when passing parameters to
/// components.
///
/// # Children
/// Children to a component come after the component's surrounded by braces (`{ ... }`).  Like parameters, children
/// are optional, but are only valid for components that accept them. For example [`Text`] accepts no children, but
/// [`Section`] does.
///
/// Children are passed as [`Children`], which have a fixed size. Components can therefore specify exactly how many
/// children they take in. Some components, like [`VStack`] and [`HStack`] take in a variable number of children,
/// while some, like [`Section`] accept a fixed number.
///
/// [`Children`]: element::Children
/// [`HStack`]: components::HStack
/// [`Section`]: components::Section
/// [`Text`]: components::Text
/// [`VStack`]: components::VStack
/// [`Component::render`]: components::Component::render
/// [`TryInto::try_into`]: https://doc.rust-lang.org/std/convert/trait.TryInto.html#tymethod.try_into
/// [SwiftUI-like]: https://developer.apple.com/xcode/swiftui/
pub use intuitive_macros::render;
