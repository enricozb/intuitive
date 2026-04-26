//! A scrollable stack of single-line entries.

use crate::{buffer::Style, macros::component, runtime::Context};

/// A scrollable stack of single-line entries.
///
/// Required:
///
/// - `items` - the lines to display, in order.
///
/// Optional:
///
/// - `.selected` - index of the highlighted row (default `0`). May exceed
///   `items.len() - 1`, in which case nothing is highlighted.
/// - `.focused` - controls highlight styling (default `false`). When focused,
///   the selected row is drawn in `selected_fg` and bold; when unfocused, it's
///   drawn bold only.
/// - `.selected_fg` - color used for the selected row when focused (default
///   `Color::Yellow`).
///
/// # Scrolling
///
/// If `items.len()` exceeds the region's height, `List` scrolls the
/// visible window so the selected row is always shown. The window is
/// computed purely from `selected` and the region height \\- there is no
/// internal scroll state.
#[component]
pub fn List(
  ctx: &mut Context<'_>,
  items: Vec<String>,
  #[default = "0"] selected: usize,
  #[default] selected_style: Style,
  #[default] default_style: Style,
) {
  let mut region = ctx.region();
  let h = region.size().h;
  if h == 0 || items.is_empty() {
    return;
  }
  let h_usize = usize::from(h);
  let n = items.len();
  // Compute the scroll offset: the smallest start such that
  // `selected` is in `[start, start + h)`.
  let start = if n <= h_usize {
    0
  } else if selected >= n {
    // Selected past the end: clamp to last viable window.
    n - h_usize
  } else if selected < h_usize {
    0
  } else {
    // selected is at least h cells from the top; show it as the
    // last visible row.
    selected + 1 - h_usize
  };
  let end = (start + h_usize).min(n);
  for (row, i) in (start..end).enumerate() {
    let line = &items[i];
    let is_selected = i == selected;
    let style = if is_selected { selected_style } else { default_style };
    let y = u16::try_from(row).unwrap_or(u16::MAX);
    region.write_str((0, y), line, style);
  }
}
