use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread,
  time::Duration,
};

use intuitive::{
  component,
  components::{Fixed, Padding, Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  render::hooks::{use_effect, use_state, Cleanup},
  style::{Color, Style},
  terminal::Terminal,
  utils::layout::{Alignment, Amount},
};

/// A component that shows how many seconds it has been rendered for.
#[component(Timer)]
fn render(text: String, border: Style) -> AnyElement {
  let seconds = use_state(|| 0);
  let seconds_clone = seconds.clone();

  use_effect(|| {
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    thread::spawn(move || {
      while !done.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));

        seconds_clone.update(|seconds| seconds + 1).unwrap();
      }
    });

    Cleanup::from(move || {
      done_clone.store(true, Ordering::Relaxed);
    })
  });

  render! {
    Padding(amount: Amount::Percentage(10)) {
      Fixed(height: Amount::Fixed(3)) {
        Section(title: "Seconds", border: border.clone()) {
          Text(text: format!("{} has been up for {} seconds", text, seconds), alignment: Alignment::Center)
        }
      }
    }
  }
}

/// Swaps between two timers.
#[component(Root)]
fn render() -> AnyElement {
  let first = use_state(|| true);
  let first_clone = first.clone();

  use_effect(|| {
    thread::spawn(move || loop {
      thread::sleep(Duration::from_secs(5));

      first_clone.update(|first| !first).unwrap();
    });
  });

  if first.get() {
    render! {
      Timer(key: 1, text: "The first component", border: Color::Green)
    }
  } else {
    render! {
      Timer(key: 1, text: "The second component", border: Color::Red)
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
