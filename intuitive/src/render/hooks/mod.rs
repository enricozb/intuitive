//! Enables state-like features in functional components.
//!
//! # Hooks
//! Hooks enable state-like behavior in otherwise functional components. They are heavily inspired by
//! [React's hooks].
//!
//! The main hooks are:
//! - [`UseState`]
//! - [`UseEffect`]
//!
//! ## Using Hooks
//! Hooks are implemented as traits on the [`Hooks`] provider, which has a primitive [`use_hook`] hook.
//!
//! In order to use hooks:
//!   1. Import the hook's trait
//!   2. Call the function on the [`Hooks`] provider.
//!      - This is implictly introduced into scope as `hooks` when using the [`#[component(..)]`](crate::component)
//!        attribute macro. (See the [hooks section](crate::component#hooks) of the attribute macro's documentation).
//!
//! For instance, an example usage of the [`UseState`] and [`UseEffect`] hooks,
//! ```rust
//! use std::{thread, time::Duration};
//!
//! use intuitive::{
//!   component,
//!   components::{Section, Text},
//!   element::Any as AnyElement,
//!   render,
//!   render::hooks::{UseEffect, UseState},
//!   style::Color,
//! };
//!
//! #[component(Root)]
//! fn render() -> AnyElement {
//!   let seconds = hooks.use_state(|| 0);
//!
//!   // cloned because it's moved into the `use_effect` hook below
//!   let seconds_clone = seconds.clone();
//!
//!   hooks.use_effect(|| {
//!     thread::spawn(move || loop {
//!       thread::sleep(Duration::from_secs(1));
//!
//!       seconds_clone.update(|seconds| seconds + 1).unwrap();
//!     });
//!   });
//!
//!   render! {
//!     Section(title: "Seconds", border: Color::Red) {
//!       Text(text: format!("This program has run for {} seconds", seconds.get()))
//!     }
//!   }
//! }
//! ```
//!
//! ## Writing Custom Hooks
//! Custom hooks are, like the built-in hooks, implemented as traits on the [`Hooks`] context provider. For example,
//! the [`UseState`] hook is implemented as follows:
//! ```rust
//! pub trait UseState {
//!   fn use_state<F, T>(&mut self, initializer: F) -> State<T>
//!   where
//!     F: FnOnce() -> T,
//!     T: 'static;
//! }
//!
//! impl UseState for Hooks {
//!   fn use_state<F, T>(&mut self, initializer: F) -> State<T>
//!   where
//!     F: FnOnce() -> T,
//!     T: 'static,
//!   {
//!     self
//!       .use_hook(|component_id| State::new(component_id, initializer()).into())
//!       .expect("use_state: use_hook")
//!   }
//! }
//! ```
//! Now, when [`UseState`] is imported, [`UseState::use_state`] can be called as a method on [`Hooks`]. Custom
//! hooks can also rely on higher-level built in hooks by importing their respective traits.
//!
//! ### `Hooks::use_hook`
//! This is the "primitive" hook that all other hooks must eventually use in order to hook into the component
//! rendering lifecycle. This is the only hook that is not implemented as a trait, and is therefore always available.
//! See the [method's documentation](crate::render::providers::Hooks::use_hook) for details.
//!
//! [`Component::render`]: crate::components::Component::render
//! [`Context`]: crate::render::Context
//! [`Context::hooks`]: crate::render::Context::hooks
//! [`Hooks`]: crate::render::providers::Hooks
//! [`use_hook`]: crate::render::providers::Hooks::use_hook
//! [React's hooks]: https://reactjs.org/docs/hooks-intro.html

pub mod error;
mod use_effect;
mod use_effect_with_deps;
mod use_state;

use std::any::{self, Any};

use self::error::{Error, Result};
pub use self::{
  use_effect::{Cleanup, UseEffect},
  use_effect_with_deps::UseEffectWithDeps,
  use_state::{State, UseState},
};

/// A dynamically-typed hook return value, along with a deinitialization function for unmounting.
pub struct Hook {
  /// The inner value of the hook.
  value: Box<dyn Any>,

  /// Any deinitialization needed for whne this hook's parent component is unmounted.
  deinit: Option<Box<dyn FnOnce()>>,
}

impl Default for Hook {
  fn default() -> Self {
    Self {
      value: Box::new(()),
      deinit: None,
    }
  }
}

impl Hook {
  /// Creates a new [`Hook`].
  #[must_use]
  pub fn new<T, F>(value: T, deinit: F) -> Self
  where
    T: 'static,
    F: 'static + FnOnce(),
  {
    Self {
      value: Box::new(value),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Creates a new [`Hook`] with only a value, and no deinitialization function.
  pub fn from_value<T>(value: T) -> Self
  where
    T: 'static,
  {
    Self {
      value: Box::new(value),
      deinit: None,
    }
  }

  /// Creates a new [`Hook`] with only a deinitialization function, and unit value.
  pub fn from_deinit<F>(deinit: F) -> Self
  where
    F: 'static + FnOnce(),
  {
    Self {
      value: Box::new(()),
      deinit: Some(Box::new(deinit)),
    }
  }

  /// Calls the `deinit` function
  pub fn deinit(self) {
    if let Some(deinit) = self.deinit {
      deinit();
    }
  }

  /// Calls [`Any.downcast_ref`] on the [`Hook`]s inner value.
  ///
  /// # Errors
  ///
  /// Will return an `Err` if the hook's value can't be cast to `T`.
  pub fn downcast_ref<T: 'static + Clone>(&self) -> Result<T> {
    Ok(self.value.downcast_ref::<T>().ok_or(Error::InvalidType(any::type_name::<T>()))?.clone())
  }
}
