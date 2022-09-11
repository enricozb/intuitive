//! Contains the `Terminal` type used to run the UI.

use std::io::{self, Stdout};

use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
pub use tui::layout::Rect;
use tui::{backend::CrosstermBackend, terminal::Frame as TuiFrame, Terminal as TuiTerminal};

use crate::{
  components::Any as AnyComponent,
  element::Any as AnyElement,
  error::Result,
  event::{self, Event},
  state,
};

pub type Backend = CrosstermBackend<Stdout>;
pub type Frame<'a> = TuiFrame<'a, Backend>;

pub struct Terminal {
  root: AnyComponent,
  terminal: TuiTerminal<Backend>,
}

impl Terminal {
  pub fn new(root: AnyComponent) -> Result<Self> {
    Self::setup()?;

    event::start_crossterm_events();

    ctrlc::set_handler(event::quit).expect("setting sigterm handler");

    Ok(Self {
      root,
      terminal: TuiTerminal::new(CrosstermBackend::new(io::stdout()))?,
    })
  }

  fn setup() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    Ok(())
  }

  fn cleanup(&mut self) -> Result<()> {
    disable_raw_mode()?;
    execute!(self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    self.terminal.show_cursor()?;

    Ok(())
  }

  fn draw(&mut self, element: &AnyElement) -> Result<()> {
    self.terminal.draw(|frame| {
      element.draw(frame.size(), frame);
    })?;

    Ok(())
  }

  pub fn run(&mut self) -> Result<()> {
    let component = self.root.render();
    state::render_done();

    self.draw(&component)?;

    loop {
      let component = self.root.render();
      state::render_done();

      match event::read()? {
        Event::Render => self.draw(&component)?,
        Event::Key(event) => component.on_key(event),
        Event::Quit => break,
      }
    }

    Ok(())
  }
}

impl Drop for Terminal {
  fn drop(&mut self) {
    self.cleanup().unwrap();
  }
}
