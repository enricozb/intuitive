//! Layout containers for horizontal/vertical stacking and centering.
//!
//! `HStack`, `VStack`, `ZStack`, and `Centered`: containers that split
//! their region between their composed children by per-slot [`Flex`]
//! rules.
//!
//! Each child slot is described by a [`Flex`] entry:
//!
//! - `Flex::Frac(w)` claims a proportional share of the space left over after
//!   all `Flex::Cell` slots are placed.
//! - `Flex::Cell(n)` reserves exactly `n` cells along the split axis.
//!
//! Cell allocations are clipped (never panic) if the parent region is
//! too small to satisfy them. See [`Rectangle::split_h_flex`] for the
//! exact semantics.
//!
//! [`Flex`]: crate::geometry::Flex
//! [`Rectangle::split_h_flex`]: crate::geometry::Rectangle::split_h_flex

use crate::{
  geometry::Flex,
  macros::component,
  runtime::{Context, Element},
};

/// Horizontal stack: splits its rect along the x-axis according to
/// `flex`, rendering each composed child into one slot.
///
/// `flex` and the composed-children count must match each render;
/// mismatches panic in debug builds.
#[component]
pub fn HStack(ctx: &mut Context<'_>, #[default] flex: Vec<Flex>, #[children] children: Vec<Element>) {
  let rect = ctx.region().rect();
  let flex = if flex.is_empty() {
    &vec![Flex::Frac(1); children.len()]
  } else {
    &flex
  };
  let parts = rect.split_h_flex(flex);
  debug_assert_eq!(parts.len(), children.len(), "flex and children counts must match");
  for (sub_rect, child) in parts.into_iter().zip(children) {
    ctx.render(child, sub_rect);
  }
}

/// Vertical stack: same as `HStack` but splits along the y-axis.
#[component]
pub fn VStack(ctx: &mut Context<'_>, #[default] flex: Vec<Flex>, #[children] children: Vec<Element>) {
  let rect = ctx.region().rect();
  let flex = if flex.is_empty() {
    &vec![Flex::Frac(1); children.len()]
  } else {
    &flex
  };
  let parts = rect.split_v_flex(flex);
  debug_assert_eq!(parts.len(), children.len(), "flex and children counts must match");
  for (sub_rect, child) in parts.into_iter().zip(children) {
    ctx.render(child, sub_rect);
  }
}

/// Depth stack: same as `HStack` but splits along the z-axis. Later children
/// appear on top of earlier ones. For example, in
/// ```ignore
/// ui!{
///   ZStack() {
///     A()
///     B()
///   }
/// }
/// ```
/// `A()` will be rendered on behind `B()`.
///
/// `ZStack` is the canonical overlap-producing container: each
/// composed child is placed at a strictly increasing z (the i-th
/// child is placed at `ctx.z() + 1 + i`). The +1 offset ensures the
/// first child still sits *above* the `ZStack`'s own background
/// (which it doesn't paint, but which other siblings of the
/// `ZStack` itself might), so a nested `ZStack { Inner, Overlay }`
/// inside an outer `ZStack { Base, ZStack {..} }` produces a
/// monotonic z-sequence across the whole tree. The runtime uses
/// the per-child z to (a) order partial re-renders so a higher-z
/// overlay always repaints over a lower-z base, and (b) skip the
/// overlap-walk for siblings at the same z.
#[component]
pub fn ZStack(ctx: &mut Context<'_>, #[children] children: Vec<Element>) {
  let rect = ctx.region().rect();
  let base_z = ctx.z();
  for (i, child) in children.into_iter().enumerate() {
    // `i + 1` keeps each child strictly above the ZStack itself.
    // `try_from` is essentially infallible here (a Vec with `i32::MAX + 1`
    // children would have exhausted memory long before this point);
    // saturate on the off chance.
    let offset = i32::try_from(i + 1).unwrap_or(i32::MAX);
    let child_z = base_z.saturating_add(offset);
    ctx.render_with_z(child, rect, child_z);
  }
}

/// Centered container: positions a single composed child inside a
/// configurable margin on each side of its rect.
///
/// The component splits its rect into a 3x3 grid using
/// [`Rectangle::split_h_flex`] for the columns (with flex
/// `[pad_x, width, pad_x]`) and [`Rectangle::split_v_flex`] for the
/// rows (with flex `[pad_y, height, pad_y]`), and renders its only
/// child into the center cell of that grid. The outer eight cells
/// are left untouched. The same `Flex` rule is used on both sides of
/// each axis, so the child is always centered.
///
/// All four `Flex` props default to `Flex::Frac(1)`, which produces
/// a child occupying the middle third along each axis (since the
/// three weights `[Frac(1), Frac(1), Frac(1)]` divide the available
/// extent into thirds).
///
/// - `pad_x` / `pad_y` control the *padding* on each side of the child. Setting
///   `pad_x: Flex::Cell(0)` removes horizontal padding entirely, so the child
///   spans the full width with the `width` rule applied to that full extent
///   (typically a no-op when `width` is the lone `Frac`).
/// - `width` / `height` control the size of the *child* slot. For example,
///   `Flex::Cell(40)` reserves exactly 40 cells for the child along that axis
///   (centered, padded by the resolved `pad_x` / `pad_y` on either side).
///
/// # Panics
///
/// On render, panics if the number of composed children is not
/// exactly one.
///
/// [`Rectangle::split_h_flex`]: crate::geometry::Rectangle::split_h_flex
/// [`Rectangle::split_v_flex`]: crate::geometry::Rectangle::split_v_flex
#[component]
pub fn Centered(
  ctx: &mut Context<'_>,
  #[default = "Flex::Frac(1)"] pad_x: Flex,
  #[default = "Flex::Frac(1)"] pad_y: Flex,
  #[default = "Flex::Frac(1)"] width: Flex,
  #[default = "Flex::Frac(1)"] height: Flex,
  #[children] children: Vec<Element>,
) {
  // Enforce exactly one child via the `Vec<T> -> [T; N]` conversion.
  let [child]: [Element; 1] = children
    .try_into()
    .unwrap_or_else(|v: Vec<Element>| panic!("Centered accepts exactly one child, got {}", v.len()));
  let rect = ctx.region().rect();
  // Reuse the 1-D `Flex` splitter that `HStack` / `VStack` are built on:
  // first carve out the middle column (sized by `width`, padded by
  // `pad_x` on each side), then carve the middle row (sized by
  // `height`, padded by `pad_y` on each side) out of that column.
  let cols = rect.split_h_flex(&[pad_x, width, pad_x]);
  let middle_col = cols[1];
  let rows = middle_col.split_v_flex(&[pad_y, height, pad_y]);
  let center = rows[1];
  ctx.render(child, center);
}
