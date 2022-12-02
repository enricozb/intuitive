use std::io::{self, Stdout};

use crossterm::{
  cursor::{Hide as HideCursor, Show as ShowCursor},
  execute,
  terminal::{self, Clear, ClearType},
};

#[allow(unused)]
use crate::element::Element;
use crate::{
  buffer::{region::Region, Buffer},
  element::Any as AnyElement,
  error::Result,
  event::{self, Event},
  render,
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
    self.draw(element)?;

    loop {
      match event::read()? {
        Event::Rerender(component_id) => {
          render::rerender(component_id)?;
          self.draw(element)?;
        }

        Event::Resize => {
          self.resize()?;
          execute!(&self.stdout, Clear(ClearType::All))?;
          self.draw(element)?;
        }

        Event::Quit => break,
      }
    }

    Ok(())
  }

  /// Recomputes the current size of the terminal.
  fn resize(&mut self) -> Result<()> {
    let size = terminal::size()?;
    self.buffers = [Buffer::new(size), Buffer::new(size)];

    Ok(())
  }

  /// Returns a the current [`Region`].
  fn current_region(&mut self) -> Region {
    (&mut self.buffers[usize::from(self.idx)]).into()
  }

  /// Draw the provided [`Element`] onto the terminal.
  fn draw(&mut self, element: &AnyElement) -> Result<()> {
    element.draw(&mut self.current_region())?;
    self.paint_diffs()?;

    Ok(())
  }

  /// Draw the differences between the current and previous [`Buffer`]s onto [`Self::stdout`].
  fn paint_diffs(&mut self) -> Result<()> {
    let current_buffer = &self.buffers[usize::from(self.idx)];
    let previous_buffer = &self.buffers[usize::from(!self.idx)];

    current_buffer.paint_diffs(previous_buffer, &mut self.stdout)?;

    self.idx = !self.idx;

    Ok(())
  }

  /// Prepares the [`Terminal`] for rendering.
  fn prepare(&self) -> Result<()> {
    execute!(&self.stdout, terminal::EnterAlternateScreen)?;
    execute!(&self.stdout, Clear(ClearType::All))?;
    execute!(&self.stdout, HideCursor)?;

    terminal::enable_raw_mode()?;
    event::start_crossterm_events();

    Ok(())
  }

  /// Cleans up the [`Terminal`] after rendering.
  fn cleanup(&self) -> Result<()> {
    terminal::disable_raw_mode()?;

    execute!(&self.stdout, ShowCursor)?;
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
