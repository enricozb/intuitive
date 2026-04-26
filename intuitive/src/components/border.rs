//! Draws a 1-cell-thick border around its child, with an optional title.

use crate::{
  buffer::{Cell, Style},
  geometry::Size,
  macros::component,
  runtime::{Context, Element},
};

/// Draws a 1-cell-thick border around its child, with an optional title.
///
/// If more than one child is provided, only the first is drawn.
#[component]
pub fn Border(
  ctx: &mut Context<'_>,
  #[default] title: String,
  #[default] style: Style,
  #[children] children: Vec<Element>,
) {
  let rect = ctx.region().rect();

  ctx.set_opaque();

  let mut region = ctx.region();
  let Size { w, h } = region.size();

  if w == 0 || h == 0 {
    return;
  }

  region.fill(Cell::BLANK);

  let nw = Cell::styled('┌', style);
  let ne = Cell::styled('┐', style);
  let se = Cell::styled('┘', style);
  let sw = Cell::styled('└', style);
  let tb = Cell::styled('─', style);
  let lr = Cell::styled('│', style);

  if w >= 2 && h >= 2 {
    region.set((0, 0), nw);
    region.set((w - 1, 0), ne);
    region.set((0, h - 1), sw);
    region.set((w - 1, h - 1), se);

    for x in 1..w - 1 {
      region.set((x, 0), tb);
      region.set((x, h - 1), tb);
    }

    for y in 1..h - 1 {
      region.set((0, y), lr);
      region.set((w - 1, y), lr);
    }
  }

  if !title.is_empty() && w >= 4 {
    let max_chars = w.saturating_sub(4) as usize;
    let end = title.char_indices().nth(max_chars).map_or(title.len(), |(i, _)| i);

    region.write_str((2, 0), &title[..end], style);
  }

  // TODO(enricozb): i think this unmounts the child if the region is too small
  let inner = rect.inset();

  if inner.size.w == 0 || inner.size.h == 0 {
    return;
  }

  if let Some(child) = children.pop() {
    ctx.render(child, inner);
  }
}
