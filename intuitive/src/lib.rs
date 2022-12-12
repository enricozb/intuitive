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

pub use intuitive_macros::component;
/// Macro for rendering components.
///
/// # Usage
/// This macro is used to render components from within [`Component::render`]. It uses a [SwiftUI]-like syntax, where
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
/// [SwiftUI]: https://developer.apple.com/xcode/swiftui/
pub use intuitive_macros::render;
