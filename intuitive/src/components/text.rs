//! A single-line text component.

use crate::{
  buffer::Style,
  geometry::{Align, Size},
  macros::component,
  runtime::Context,
};

/// A single-line text component.
///
/// `text` is required; everything else is optional with sensible defaults:
///
/// - `.fg` - foreground color (default `Color::Reset`).
/// - `.bold` - boldness flag (default `false`).
/// - `.align` - one of the nine [`Align`] points; defaults to `Align::NW`
///   (top-left), matching the historical behavior. Use e.g. `Align::Center` to
///   draw centered in the region, or `Align::S` to anchor to the bottom-center.
#[component]
pub fn Text(ctx: &mut Context<'_>, text: String, #[default] style: Style, #[default] align: Align) {
  let mut region = ctx.region();
  if text.is_empty() {
    return;
  }
  let container = region.size();
  let text_w = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
  let pos = align.place(Size::new(text_w, 1), container);
  region.write_str(pos, &text, style);
}
