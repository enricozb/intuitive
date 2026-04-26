//! A single-line text input with an internal cursor.

use crate::{
  buffer::{Cell, Color, Style},
  event::{KeyCode, KeyEventKind, KeyModifiers},
  macros::component,
  runtime::Context,
  signal::Signal,
};

/// A single-line text input with an internal cursor.
///
/// `Input` is intentionally minimal: it draws no borders. Wrap it in a
/// [`Border`] (or any other container) if you want a frame around it.
///
/// Props:
///
/// - `value` (required) — the input's text. `Input` reads via `value.get(ctx)`
///   (subscribing the element) and writes via `value.update(..)` from inside
///   its key handler. Hosting code can observe the user's typing by subscribing
///   to the same signal.
/// - `focused` (required) — a boolean signal that gates focus. When it reads
///   `true`, `Input` draws a cursor and installs a key handler that mutates
///   `value`; when `false`, neither the cursor nor the handler is registered,
///   so other components own the keyboard.
///
/// The cursor position is per-element state (a `Signal<usize>`
/// created with `ctx.new_signal`). It is not exposed as a prop; it
/// survives across renders as long as the `Input` element stays
/// mounted, and is reset to zero when it remounts. The cursor is
/// clamped to the character length of `value` on each render, so it
/// remains valid even if the value is mutated from outside the
/// component.
///
/// # Scrolling
///
/// When the text is wider than the region, `Input` horizontally
/// scrolls so the cursor stays visible. The scroll offset (the
/// character index of the leftmost visible character) is per-element
/// state, adjusted minimally each render: it only moves when the
/// cursor would otherwise fall outside `[offset, offset + w)`. There
/// is no scrollbar — the visible window simply slides under the
/// cursor.
///
/// Key bindings (only active when `focused` reads `true`):
///
/// - `Char(c)` (no modifiers) — insert `c` at the cursor and advance.
/// - `Backspace` — delete the character before the cursor.
/// - `Delete` — delete the character at the cursor.
/// - `Left` / `Right` — move the cursor.
/// - `Home` / Ctrl-A — jump to the start of the text.
/// - `End` / Ctrl-E — jump to the end of the text.
///
/// Other modifier combinations on `Char` (e.g. Ctrl-X, Alt-foo) are
/// ignored so they don't get inserted as literal characters.
///
/// [`Border`]: crate::components::Border
#[component]
pub fn Input(ctx: &mut Context<'_>, value: Signal<String>, focused: Signal<bool>) {
  // `Input` fills its entire rect every render (it blanks the
  // region before drawing text and the cursor), so it is opaque.
  // Marking it opaque lets the runtime repaint this `Input` on
  // top of any lower-z element that re-renders alone underneath
  // it — e.g. a timer ticking while a modal-hosted input is
  // focused over it.
  ctx.set_opaque();

  // Per-element cursor position, measured in characters (not bytes).
  let cursor = ctx.new_signal::<usize>(0);
  // Per-element scroll offset: the character index of the leftmost
  // cell that's currently visible. Like `cursor`, it persists across
  // renders and is adjusted below to keep the cursor on-screen.
  let scroll = ctx.new_signal::<usize>(0);

  let text = value.get(ctx);
  let is_focused = focused.get(ctx);
  let len = text.chars().count();
  // Clamp the cursor to the current text length so external writes to
  // `value` cannot leave the cursor pointing past the end.
  let cur = cursor.get(ctx).min(len);

  // Snapshot region dimensions so we can compute the scroll offset
  // before drawing. `ctx.region()` is just a sub-view of the back
  // buffer; reading its size does not subscribe to anything.
  let (w, h) = {
    let region = ctx.region();
    (region.size().w, region.size().h)
  };
  if w == 0 || h == 0 {
    return;
  }
  let w_usize = usize::from(w);

  // Compute the scroll offset so the cursor is always visible.
  //
  // The visible window is `[offset, offset + w)` measured in
  // characters. We want `cur` to fall inside it. If the previous
  // offset still satisfies that, keep it (minimal movement — typing
  // in the middle of a long line shouldn't cause unrelated
  // scrolling). Otherwise slide just far enough:
  //
  //   - cursor before window → anchor window at the cursor (`offset = cur`).
  //   - cursor at or past window's right edge → anchor so the cursor sits one
  //     cell before the right edge (`offset = cur + 1 - w`). That "one cell
  //     before" choice leaves a column for the cursor block when typing at the
  //     end of an overflowing string.
  //
  // After this, snap the offset down so the window never extends
  // past the end of the text unless the cursor itself sits past the
  // existing characters: e.g. deleting until the text fits should
  // restore `offset = 0`.
  let prev_offset = scroll.peek().min(len);
  let raw_offset = if cur < prev_offset {
    cur
  } else if cur + 1 > prev_offset + w_usize {
    cur + 1 - w_usize
  } else {
    prev_offset
  };
  // Upper bound: we shouldn't scroll past `len + 1 - w` (the offset
  // at which the last char is on-screen with one trailing cursor
  // cell), and we shouldn't scroll past `cur` (which would push the
  // cursor off the left edge).
  let max_offset = (len + 1).saturating_sub(w_usize).min(cur);
  let new_offset = raw_offset.min(max_offset);
  if new_offset != scroll.peek() {
    scroll.set(new_offset);
  }

  // Draw the text and, if focused, the cursor.
  {
    let mut region = ctx.region();
    // Blank our region first so we draw onto an opaque background.
    // Without this, when `Input` is mounted over other content (e.g.
    // inside a `Modal` overlay), cells past the end of `text` would
    // leak through whatever the underlying layer drew there.
    region.fill(Cell::BLANK);

    // Slice out the visible character window. We index by characters
    // (not bytes) so the slice aligns with the cursor / offset units.
    let visible: String = text.chars().skip(new_offset).take(w_usize).collect();
    region.write_str((0, 0), &visible, Style::RESET);

    if is_focused {
      // The cursor's screen column is its char-index minus the scroll
      // offset. `new_offset` was chosen so this falls inside `[0, w)`.
      let screen_x = cur.saturating_sub(new_offset);
      let cursor_x = u16::try_from(screen_x).unwrap_or(u16::MAX);
      if cursor_x < w {
        // Render the character under the cursor on a dark-grey background
        // so the cursor reads as a colored block. When the cursor is at
        // the end of the text there is no character to highlight, so we
        // paint a blank cell with the same background to mark the
        // insertion point.
        let ch = text.chars().nth(cur).unwrap_or(' ');
        region.set((cursor_x, 0), Cell::styled(ch, Style::new().bg(Color::DarkGrey)));
      }
    }
  }

  // Only install the key handler when focused. Skipping registration on
  // unfocused renders lets the runtime's end-of-render GC drop the prior
  // handler slot, so a stale closure cannot react to events while
  // another component is logically holding focus.
  if is_focused {
    let value_sig = value.clone();
    let cursor_sig = cursor.clone();
    ctx.on_key(move |key| {
      if key.kind != KeyEventKind::Press {
        return;
      }
      // Readline-style cursor shortcuts. Match these before the
      // generic `Char(c)` arm below so a Ctrl-A keystroke jumps to
      // the start of the line instead of inserting a literal 'a'.
      let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
      match key.code {
        KeyCode::Char('a') if ctrl => {
          cursor_sig.set(0);
          return;
        }
        KeyCode::Char('e') if ctrl => {
          let cur_len = value_sig.peek().chars().count();
          cursor_sig.set(cur_len);
          return;
        }
        _ => {}
      }
      match key.code {
        // Only insert when no "command" modifier is held. Shift is
        // fine — it's part of the user typing an uppercase character.
        KeyCode::Char(c)
          if !(key.modifiers.contains(KeyModifiers::CONTROL) || key.modifiers.contains(KeyModifiers::ALT)) =>
        {
          let pos = cursor_sig.peek();
          value_sig.update(|v| {
            let byte_idx = v.char_indices().nth(pos).map_or_else(|| v.len(), |(i, _)| i);
            v.insert(byte_idx, c);
          });
          cursor_sig.update(|p| *p += 1);
        }
        KeyCode::Backspace => {
          let pos = cursor_sig.peek();
          if pos > 0 {
            value_sig.update(|v| {
              if let Some((byte_idx, _)) = v.char_indices().nth(pos - 1) {
                v.remove(byte_idx);
              }
            });
            cursor_sig.update(|p| *p -= 1);
          }
        }
        KeyCode::Delete => {
          let pos = cursor_sig.peek();
          value_sig.update(|v| {
            if let Some((byte_idx, _)) = v.char_indices().nth(pos) {
              v.remove(byte_idx);
            }
          });
        }
        KeyCode::Left => {
          cursor_sig.update(|p| *p = p.saturating_sub(1));
        }
        KeyCode::Right => {
          let cur_len = value_sig.peek().chars().count();
          cursor_sig.update(|p| *p = (*p + 1).min(cur_len));
        }
        KeyCode::Home => {
          cursor_sig.set(0);
        }
        KeyCode::End => {
          let cur_len = value_sig.peek().chars().count();
          cursor_sig.set(cur_len);
        }
        _ => {}
      }
    });
  }
}
