use std::io::{self, Stdout};

use crossterm::{execute, terminal};

use crate::{buffer::Buffer, element::Any as AnyElement, error::Result};

/// A terminal that can be drawn onto.
pub struct Terminal {
  /// The [`Stdout`] to write to.
  stdout: Stdout,
  /// The two buffers that are used for drawing.
  buffers: [Buffer; 2],
  /// The current index of the [`Buffer`] being drawn onto.
  idx: bool,
}

impl Terminal {
  /// Creates a new [`Terminal`].
  pub fn new() -> Result<Self> {
    let size = terminal::size()?;

    Ok(Self {
      stdout: io::stdout(),
      buffers: [Buffer::new(size), Buffer::new(size)],
      idx: false,
    })
  }

  /// Starts the render loop, given a root [`AnyElement`].
  pub fn render(&self, element: &AnyElement) -> Result<()> {
    self.prepare()?;

    Ok(())
  }

  /// Prepares the [`Terminal`] for rendering.
  fn prepare(&self) -> Result<()> {
    execute!(&self.stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    Ok(())
  }

  /// Cleans up the [`Terminal`] after rendering.
  fn cleanup(&self) -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(&self.stdout, terminal::LeaveAlternateScreen)?;

    Ok(())
  }
}

/// Calls [`Terminal::cleanup`] to restore the terminal's state and screen before dropping.
#[allow(rustdoc::private_intra_doc_links)]
impl Drop for Terminal {
  fn drop(&mut self) {
    self.cleanup().expect("Terminal::cleanup")
  }
}
