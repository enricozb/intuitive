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
//! TODO(enricozb): recipes section

pub mod children;
pub mod modal;

mod any;
mod centered;
mod embed;
mod empty;
mod section;
mod stack;
mod table;
mod text;

pub use self::{
  any::Any,
  centered::Centered,
  embed::Embed,
  empty::Empty,
  section::Section,
  stack::{Flex, Horizontal as HStack, Vertical as VStack},
  table::Table,
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
/// 1. All structures implementing `Component` that will ever be rendered must be
///    constructed before calling `Terminal::run()`.
/// 2. When rendering a frame, [`Component::render`] must be called on every `Component`, even if it
///    is not being drawn this frame.
///    - This is to ensure that hooks, such as [`use_state`], are always called in the
///      same order.
///    - This can typically be guaranteed by always calling [`Component::render`]
///      on your component's children.
/// 3. [`Component::render`] must never be called outside of [`Component::render`]. This is to
///    continue the assurances made in the previous point.
/// 4. Structures implementing `Component`, must also implement `Default`.
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
