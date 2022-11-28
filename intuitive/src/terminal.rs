use std::io::{self, Stdout};

use crossterm::{execute, terminal};

use crate::{
  buffer::{region::Region, Buffer},
  element::Any as AnyElement,
  error::Result,
  event::{self, Event},
};

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
  ///
  /// # Errors
  ///
  /// Will return `Err` if the terminal's size cannot be read from [`Stdout`].
  pub fn new() -> Result<Self> {
    let size = terminal::size()?;

    Ok(Self {
      stdout: io::stdout(),
      buffers: [Buffer::new(size), Buffer::new(size)],
      idx: false,
    })
  }

  /// Starts the render loop, given a root [`AnyElement`].
  ///
  /// # Errors
  ///
  /// Will return `Err` if [`Terminal::prepare`] fails.
  #[allow(rustdoc::private_intra_doc_links)]
  pub fn render(&mut self, element: &AnyElement) -> Result<()> {
    self.prepare()?;

    loop {
      match event::read()? {
        Event::Resize => element.draw(self.current_region())?,
      }

      // TODO(enricozb): only do this when the current buffer is dirty
      self.draw_diffs()?;
    }

    Ok(())
  }

  /// Returns a the current [`Region`].
  fn current_region(&mut self) -> Region {
    (&mut self.buffers[self.idx as usize]).into()
  }

  /// Draw the differences between the current and previous [`Buffer`]s onto [`Self::stdout`].
  fn draw_diffs(&mut self) -> Result<()> {
    let current_buffer = &self.buffers[self.idx as usize];
    let previous_buffer = &self.buffers[self.idx as usize];

    current_buffer.draw_diff(previous_buffer, &mut self.stdout)?;

    Ok(())
  }

  /// Prepares the [`Terminal`] for rendering.
  fn prepare(&self) -> Result<()> {
    execute!(&self.stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    event::start_crossterm_events();

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
    self.cleanup().expect("Terminal::cleanup");
  }
}
