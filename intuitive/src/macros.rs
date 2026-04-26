//! [`component`] and [`ui!`] macros for defining and consuming components.
//!
//! # Overview
//!
//! Components are defined using the [`component`] attribute and referenced in
//! the [`ui!`] macro. For example:
//! ```
//! # use intuitive::{
//! #   buffer::Style,
//! #   components::{Border, Text},
//! #   macros::{component, ui},
//! #   runtime::Element,
//! # };
//! #[component]
//! pub fn User(name: String, age: usize, #[default] style: Style) -> Element {
//!   ui! {
//!     Border(.title: name, .style: style) {
//!       Text(text: age.to_string())
//!     }
//!   }
//! }
//! ```
//!
//! # Defining Components
//!
//! As shown above, components are defined using the [`component`] attribute
//! attached to a function. The top-level attribute takes no arguments, and
//! can only be applied to a function.
//!
//! ## Optional Properties
//!
//! Properties of a component can be optional values by being annotated with
//! the `#[default]` attribute. This annotation can also take a value which is
//! the expression which is to be used if the property is not provided. For
//! example,
//!
//! ## Expansion & Builder Pattern
//!
//! The [`component`] attribute macro expands to two structures, the first
//! specifies the required parameters and the second all parameters.
//!
//! For example,
//! ```
//! # use intuitive::{runtime::Element, macros::component};
//! #[component]
//! pub fn Example(required: usize, #[default] optional: usize) -> Element {
//!   todo!("example body")
//! }
//! ```
//! expands to something like
//! ```
//! # use intuitive::{runtime::Element, macros::component};
//! pub struct Example<required = usize> {
//!   required: required,
//! }
//!
//! impl<required> Example<required>
//! where
//!   required: Into<usize>
//! {
//!   pub fn new(self) -> ExampleBuilder {
//!     ExampleBuilder {
//!       required: self.required.into(),
//!       optional: Default::default(),
//!     }
//!   }
//! }
//!
//! pub struct ExampleBuilder {
//!   required: usize,
//!   optional: usize,
//! }
//!
//! impl ExampleBuilder {
//!   pub fn build(self) -> Element {
//!     let Self { required, optional } = self;
//!     todo!("example body")
//!   }
//!   pub fn optional<V>(mut self, value: V) -> Self
//!   where
//!     V: Into<usize>
//!   {
//!     self.optional = value.into();
//!     self
//!   }
//! }
//! ```
//! The two most important things of note are:
//! 1. The expanded component is generic on all required properties. This is so
//!    properties [automatically have `Into` applied to them][#auto-into] in the
//!    [`ui!`] macro.
//! 2. Required properties are provided directly to the component, and optional
//!    fields are set via method calls.
//!
//! ## Kinds of Components
//!
//! Components have three distinct kinds of definitions. It's important to
//! understand these as they affect what the [`component`] attribute macro
//! generates, and thus affects how borrow checker appears to behave when
//! checking the body of a component.
//!
//! 1. *Compositional*: no [`Context`] parameter, returns [`Element`].
//!     - This component has no state, and defers to the returned element to
//!       draw.
//!     - It **can move** its parameters/properties into the returned element.
//! 2. *Stateful*: a [`Context`] parameter, returns [`Element`].
//!     - This component may have state, and defers to the returned element to
//!       draw. It can also draw to the region if it chooses to.
//!     - It **cannot move** its parameters/properties into the returned
//!       element, and therefore must clone them beforehand.
//! 3. *Drawing*: a [`Context`] parameter, returns `()`.
//!     - It is responsible for drawing the cells to the region in context.
//!
//! ### Compositional Components
//!
//! A compositional component like
//! ```
//! # use intuitive::{components::Empty, runtime::Element, macros::{component, ui}};
//! #[component]
//! pub fn Example(prop: usize) -> Element {
//!   todo!("example body")
//! }
//! ```
//! Has its [builder]'s `build` method expand to something equivalent to
//! ```
//! # use intuitive::runtime::Element;
//! # struct ExampleBuilder { prop: () };
//!
//! impl ExampleBuilder {
//!   pub fn build(self) -> Element {
//!     let Self { prop } = self;
//!     todo!("example body")
//!   }
//! }
//! ```
//! Thus, since `ExampleBuilder::build` consumes `self`, it can move its
//! properties into its returned value.
//!
//! [builder]: #expansion--builder-pattern
//! [`Context`]: intuitive::runtime::Context
//! [`Element`]: intuitive::runtime::Element

/// Attribute macro that turns a function definition into a component
/// type with `new`, builder-style optional-arg setters, and `build`.
/// See `macros.md` for the lowering.
pub use intuitive_macros::component;
/// Declarative element-tree builder. See `macros.md` for the surface
/// syntax and lowering.
pub use intuitive_macros::ui;
