//! A collection of basic components.
//!
//! This module contains two main things:
//! - A collection of commonly used components
//! - The [`Component`] trait
//!
//! # Components
//! Components are the main building blocks of Intuitive TUIs. Most components
//! can be built using the [`component` attribute macro]. For more complex components,
//! such as those that require special handling when drawing, consider implementing
//! the [`Component`] trait directly.
//!
//! [`Component`]: trait.Component.html
//! [`component` attribute macro]: ../attr.component.html
//!
//! # Recipes
//! The examples below are a few recipes for commonly constructed components. Also be
//! sure to refer to the [examples] directory in the repository. These recipes exclude
//! the `use` statements in order to shorten the code samples.
//! - [Input Box] -- An input box
//! - [Input Box With Cursor] -- An input box with a cursor
//! - [Focus] -- How to focus on different sections
//!
//! ## Input Box
//! An input box with state can easily be created with a functional component:
//! ```rust
//! #[component(Input)]
//! fn render(title: String) {
//!   let text = use_state(|| String::new());
//!   let on_key = on_key! { [text]
//!     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
//!     KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
//!   };
//!
//!   render! {
//!     Section(title) {
//!       Text(text: text.get(), on_key)
//!     }
//!   }
//! }
//! ```
//!
//! ## Input Box With Cursor
//! Drawing a cursor requires us to implement a custom [`Element`],
//! specifically so we can control the drawing of the cursor. Notice that
//! we use a functional component to return a custom [`element::Any`], instead
//! of returning a [`render!`] invocation.
//! ```rust
//! #[component(Input)]
//! fn render(title: String) -> element::Any {
//!   let text = use_state(|| String::new());
//!
//!   let on_key = on_key! { [text]
//!     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
//!     KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
//!   };
//!
//!   element::Any::new(Frozen {
//!     cursor: text.get().len() as u16,
//!     content: render! {
//!       Section(title: title.clone(), on_key) {
//!         Text(text: text.get())
//!       }
//!     },
//!   })
//! }
//!
//! struct Frozen {
//!   cursor: u16,
//!   content: element::Any,
//! }
//!
//! impl Element for Frozen {
//!   fn on_key(&self, event: KeyEvent) {
//!     self.content.on_key(event);
//!   }
//!
//!   fn draw(&self, rect: Rect, frame: &mut Frame) {
//!     self.content.draw(rect, frame);
//!     frame.set_cursor(rect.x + self.cursor + 1, rect.y + 1);
//!   }
//! }
//! ```
//!
//! ## Focus
//! In order to implement focusing on specific sections, we need to construct the components
//! to be focused on, specifically the three `Input`s manually, when rendering our `Root` component.
//! Notice that we also call [`Component::render`] on those `Input`s, because we want to be
//! able to delegate key events to them, depending on which is focused. Lastly, we use [`Embed`]
//! in order to make use of a rendered component inside of the [`render!`] macro.
//!
//! ```rust
//! #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//! enum Focus {
//!   A,
//!   B,
//!   C,
//! }
//!
//! #[component(Input)]
//! fn render(title: String, focused: bool) {
//!   let text = use_state(|| String::new());
//!
//!   let color = if *focused { Color::Blue } else { Color::Gray };
//!
//!   let on_key = on_key! { [text]
//!     KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
//!     KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
//!   };
//!
//!   render! {
//!     Section(title, color) {
//!       Text(text: text.get(), on_key)
//!     }
//!   }
//! }
//!
//! #[component(Root)]
//! fn render() {
//!   let focus = use_state(|| Focus::A);
//!
//!   let input_a = Input::new("A".to_string(), focus.get() == Focus::A).render();
//!   let input_b = Input::new("B".to_string(), focus.get() == Focus::B).render();
//!   let input_c = Input::new("C".to_string(), focus.get() == Focus::C).render();
//!
//!   let on_key = on_key! { [focus, input_a, input_b, input_c]
//!     KeyEvent { code: Tab, .. } => focus.update(|focus| match focus {
//!       Focus::A => Focus::B,
//!       Focus::B => Focus::C,
//!       Focus::C => Focus::A,
//!     }),
//!
//!     event if focus.get() == Focus::A => input_a.on_key(event),
//!     event if focus.get() == Focus::B => input_b.on_key(event),
//!     event if focus.get() == Focus::C => input_c.on_key(event),
//!
//!     KeyEvent { code: Esc, .. } => event::quit(),
//!   };
//!
//!   render! {
//!     VStack(on_key) {
//!       Embed(element: input_a)
//!       Embed(element: input_b)
//!       Embed(element: input_c)
//!     }
//!   }
//! }
//! ```
//!
//! [examples]: TODO(enricozb):-link-to-examples
//! [`Component`]: trait.Component.html
//! [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
//! [`element::Any`]: ../element/struct.Any.html
//! [`Element`]: ../element/trait.Element.html
//! [`Embed`]: struct.Embed.html
//! [Focus]: #focus
//! [Input Box]: #input-box
//! [Input Box With Cursor]: #input-box-with-cursor
//! [`render!`]: ../macro.render.html
/// [`Component::render`]: #tymethod.render
pub mod children;
pub mod modal;
pub mod stack;

#[cfg(any(feature = "experimental", doc))]
pub mod experimental;

mod any;
mod centered;
mod embed;
mod empty;
mod section;
mod text;

pub use self::{
  any::Any,
  centered::Centered,
  embed::Embed,
  empty::Empty,
  section::Section,
  stack::{Horizontal as HStack, Vertical as VStack},
  text::Text,
};
use crate::element::Any as AnyElement;

/// A trait describing structures that can be rendered to an [`Element`].
///
/// Before implementing the `Component` trait directly, make sure that what you are trying
/// to do can't be done through the [`component` attribute macro], as there are
/// a few nuances and implicit requirements when implementing `Component`.
///
/// # Implementing `Component`
/// The general idea behind the `Component` trait is that it orchestrates the construction
/// of [`Element`]s. [`Element`]s know how to be drawn and how to handle keys.
///
/// Before implementing component, it's important to understand the implicit
/// expectations that Intuitive makes when rendering components.
///
/// ## Invariants & Expectations
/// 1. When rendering a frame, [`Component::render`] must be called on every `Component`, even if it
///    is not being drawn this frame.
///    - This is to ensure that hooks, such as [`use_state`], are always called in the
///      same order.
///    - This can typically be guaranteed by always calling [`Component::render`]
///      on your component's children.
/// 2. [`Component::render`] must never be called outside of [`Component::render`]. This is to
///    continue the assurances made in the previous point.
/// 3. Structures implementing `Component`, must also implement `Default`.
/// 4. Structures implementing `Component` must have all of their fields public.
/// 5. Structures implementing `Component` _should_ have an `on_key` parameter if they also
///    take in `children`. This `on_key` parameter should be of type [`KeyHandler`] and
///    should default to forwarding the key events to the children.
///
/// Refer to the [`Section` component source] as an example component that
/// adheres to these invariants.
///
/// ## Custom Appearance
/// In order to customize how a component is drawn by the [`Terminal`], you must
/// create a struct that implements [`Element`]. This is typically done by
/// creating two structs, one that implements `Component`, and a "frozen" struct
/// that implements [`Element`], and the one implementing `Component` returns the
/// custom [`Element`] on [`Component::render`].
///
/// Typically, when a component accepts [`Children<N>`] and returns a custom [`Element`],
/// the "frozen" structure that is constructed takes in `[AnyElement; N]` as its
/// children, because [`Component::render`] was called on the [`Children<N>`]. Again,
/// refer to the [`Section` component source] that also returns a custom [`Element`].
///
/// [`component` attribute macro]: ../attr.component.html
/// [`Terminal`]: ../terminal/struct.Terminal.html
/// [`Component::render`]: #tymethod.render
/// [`Element`]: ../element/trait.Element.html
/// [`KeyHandler`]: ../event/struct.KeyHandler.html
/// [`use_state`]: ../state/fn.use_state.html
/// [`Section` component source]: ../../src/intuitive/components/section.rs.html
/// [`Children<N>`]: children/struct.Children.html
pub trait Component {
  fn render(&self) -> AnyElement;
}
