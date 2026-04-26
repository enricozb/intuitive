//! Terminal I/O: raw-mode setup, drawing the diff between two buffers,
//! and a stream of input events from crossterm.

use std::io::{self, Write};

use crossterm::{
  cursor,
  event::{DisableMouseCapture, EnableMouseCapture},
  execute, queue,
  style::{self, Attribute, Color as CtColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
  terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
  buffer::{Buffer, Color, Style},
  geometry::Size,
};

/// Owns the terminal in alternate-screen + raw mode for the duration of
/// its life. Drop returns the terminal to its prior state.
pub struct Terminal {
  out: io::Stdout,
}

impl Terminal {
  /// Enter alternate screen + raw mode and return a handle.
  ///
  /// # Errors
  ///
  /// Returns any I/O error from enabling raw mode or entering the
  /// alternate screen.
  pub fn enter() -> io::Result<Self> {
    let mut out = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(out, EnterAlternateScreen, EnableMouseCapture, cursor::Hide)?;
    Ok(Self { out })
  }

  /// Query the terminal size.
  ///
  /// # Errors
  ///
  /// Returns any I/O error from querying the terminal.
  pub fn size(&self) -> io::Result<Size> {
    let (w, h) = terminal::size()?;
    Ok(Size::new(w, h))
  }

  /// Emit the minimal escape stream to bring the terminal in line with
  /// `curr`, given that it currently shows `prev`.
  ///
  /// Cells of `curr` that differ from `prev` are written; runs of changed
  /// cells on the same row reuse the cursor without re-positioning.
  ///
  /// If `prev.size() != curr.size()` (e.g. just after a terminal
  /// resize), the entire screen is cleared and every cell of `curr` is
  /// emitted. This is required because the terminal cells outside the
  /// previous buffer's footprint are not under our control, and
  /// trusting our diff against a partially-stale `prev` would leave
  /// terminal cruft visible in newly-revealed regions.
  ///
  /// # Errors
  ///
  /// Returns any I/O error from queueing escape sequences or flushing
  /// stdout.
  // The `last_*` locals (last_x/last_y/last_fg/last_bg/last_bold) are
  // five parallel trackers for the diff loop and intentionally share
  // their prefix; the lint isn't actionable.
  #[allow(clippy::similar_names)]
  pub fn flush(&mut self, prev: &Buffer, curr: &Buffer) -> io::Result<()> {
    let resized = prev.size() != curr.size();
    if resized {
      // Clear the entire terminal; every cell of `curr` will be
      // re-emitted by the diff loop below (which returns the full
      // buffer when sizes mismatch).
      queue!(self.out, Clear(ClearType::All))?;
    }
    let diff = curr.diff(prev);
    if diff.is_empty() {
      self.out.flush()?;
      return Ok(());
    }
    // Track last-emitted state so we can avoid redundant cursor moves /
    // color changes.
    let mut last_x: Option<u16> = None;
    let mut last_y: Option<u16> = None;
    let mut last_fg: Option<Color> = None;
    let mut last_bg: Option<Color> = None;
    let mut last_bold: Option<bool> = None;
    for (pos, cell) in diff {
      // If we're not exactly one cell to the right of the last write on
      // the same row, move the cursor.
      let need_move = !matches!((last_x, last_y), (Some(x), Some(y)) if y == pos.y && x + 1 == pos.x);
      if need_move {
        queue!(self.out, cursor::MoveTo(pos.x, pos.y))?;
      }
      let Style { fg, bg, bold } = cell.style;
      if last_fg != Some(fg) {
        queue!(self.out, SetForegroundColor(to_crossterm_color(fg)))?;
        last_fg = Some(fg);
      }
      if last_bg != Some(bg) {
        queue!(self.out, SetBackgroundColor(to_crossterm_color(bg)))?;
        last_bg = Some(bg);
      }
      if last_bold != Some(bold) {
        queue!(
          self.out,
          SetAttribute(if bold {
            Attribute::Bold
          } else {
            Attribute::NormalIntensity
          })
        )?;
        last_bold = Some(bold);
      }
      queue!(self.out, style::Print(cell.ch))?;
      last_x = Some(pos.x);
      last_y = Some(pos.y);
    }
    // Reset attributes/colors at the end so subsequent prints (e.g. on
    // exit) aren't styled.
    queue!(self.out, SetAttribute(Attribute::Reset))?;
    self.out.flush()?;
    Ok(())
  }
}

impl Drop for Terminal {
  fn drop(&mut self) {
    // Best effort; ignore errors during shutdown.
    let _ = execute!(
      self.out,
      SetAttribute(Attribute::Reset),
      DisableMouseCapture,
      cursor::Show,
      LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
    let _ = execute!(self.out, Clear(ClearType::All), cursor::MoveTo(0, 0));
  }
}

fn to_crossterm_color(c: Color) -> CtColor {
  match c {
    Color::Reset => CtColor::Reset,
    Color::Red => CtColor::Red,
    Color::Green => CtColor::Green,
    Color::Yellow => CtColor::Yellow,
    Color::Blue => CtColor::Blue,
    Color::Magenta => CtColor::Magenta,
    Color::Cyan => CtColor::Cyan,
    Color::White => CtColor::White,
    Color::DarkGrey => CtColor::DarkGrey,
  }
}
