mod component;
mod on_key;
mod render;

mod utils;

use proc_macro::TokenStream;

/// Helper attribute macro for creating functional components.
///
/// # Usage
/// This macro is used when creating functional components, where the name of
/// the generated component is the item in the attribute. For example,
/// ```rust
/// #[component(Root)]
/// pub fn render(title: String) {
///   let text = use_state(String::new);
///
///   let on_key = on_key! { [text]
///     KeyEvent { code: Char(c), .. } => text.update(|text| text.push(c)),
///     KeyEvent { code: Char(c), .. } => text.update(|text| text.pop()),
///     KeyEvent { code: Esc, .. } => event::quit(),
///   };
///
///   render! {
///     Centered() {
///       Section(title) {
///         Text(text: text.get(), on_key)
///       }
///     }
///   }
/// }
/// ```
/// constructs a `Root` component, that can be used in a [`render!`] macro.
///
/// # Parameters
/// If the `render` function contains parameters, these will become parameters to the
/// generated component. These parameters can later be supplied when using the generated
/// component in a [`render!`] macro. The parameters' types **must** implement [`Default`],
/// as the generated component derives [`Default`]. If you need more control over the
/// default values of the parameters, consider implementing the [`Component`] trait instead
/// of using the `#[component(..)]` attribute macro.
///
/// # Managing State
/// State in functional components is managed similarly to how they are in [React],
/// using the [`use_state`] hook. Refer to the [`use_state`] documentation for details.
///
/// # Handling Key Events
/// In functional components, key events are sent to the component at the root of the
/// returned [`render!`] macro invocation. This means that in the example above, the
/// key event will be sent to an instance of the [`Centered`] component. However,
/// most components forward their key events to their children (especially those that
/// have only a single child), and therefore the `on_key` handler could have been
/// provided to any of the [`Centered`], [`Section`], or [`Text`] components above.
///
/// # Generated Component
/// The generated component is a structure that implements the [`Component`] trait. It
/// also has a an associated function `new() -> component::Any` that is used to create the
/// component when passing it to `Terminal::new()`. If the component has parameters,
/// they will also be parameters to the associated function `new()`in the same order
/// they were specified in the `render` function.
///
/// # Nuances
/// There are a couple of nuances with this macro:
/// - The visibility of the generated component will be the same as that of the
///   `render` function the `#[component(..)]` attribute is applied to.
/// - The return type to `render` (and even the function name itself) are completely
///   ignored. In order to keep things consistent, it's recommended that the function
///   is called `render` and the return type is left empty.
///
/// [`Centered`]: components/struct.Centered.html
/// [`Component`]: components/trait.Component.html
/// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
/// [React]: https://reactjs.org/
/// [`render!`]: macro.render.html
/// [`Section`]: components/struct.Section.html
/// [`Text`]: components/struct.Text.html
/// [`use_state`]: state/fn.use_state.html
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
  component::parse(attr, item)
}

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
/// Parameters passed to components look like function arguments but are actually much
/// closer to structure initialization. Like struct fields, they can be passed in any
/// order, and they require the field name, unless the parameter and value are the same
/// identifier. Unlike struct fields, you can omit parameters, as any omitted parameters
/// are implicitly passed in with their default values.
///
/// ## Automatic Parameter Conversion
/// When passing parameters to components within a `render!` macro invocation, an implicit
/// [`TryInto::try_into`] call is made for each parameter. This means that you can omit
/// any `.into()` calls when passing parameters to components. This is very useful when
/// working with [`Spans`] and [`Style`], as they implement [`From`] from a variety
/// of types.
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
///
/// [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
/// [`Spans`]: spans/struct.Spans.html
/// [`Style`]: style/struct.Style.html
/// [`TryInto::try_into`]: https://doc.rust-lang.org/std/convert/trait.TryInto.html#tymethod.try_into
#[proc_macro]
pub fn render(item: TokenStream) -> TokenStream {
  render::parse(item)
}

/// Helper macro for creating key handlers.
///
/// # Details
/// This macro is used to simplify a common pattern constructing a [`KeyHandler`] where:
/// - [`event`], [`event::KeyEvent`] and [`event::KeyCode::*`] are brought into scope
/// - [`State`]s need to be cloned before being moved into the key handler
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
///
/// [`event`]: event/index.html
/// [`event::KeyEvent`]: event/struct.KeyEvent.html
/// [`event::KeyCode::*`]: event/enum.KeyCode.html
/// [`State`]: state/struct.State.html
/// [`KeyHandler`]: event/struct.KeyHandler.html
#[proc_macro]
pub fn on_key(item: TokenStream) -> TokenStream {
  on_key::parse(item)
}
