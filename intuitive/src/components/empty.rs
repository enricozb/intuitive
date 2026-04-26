//! A component that draws nothing.

use crate::{macros::component, runtime::Context};

/// A component that draws nothing.
///
/// This component is _transparent_, which means that rendering it in front of
/// another element (via [`ZStack`]), will not hide the element beneath it.
///
/// [`ZStack`]: crate::components::layout::ZStack;
#[component]
pub fn Empty(ctx: &mut Context<'_>) {
  let _ = ctx;
}
