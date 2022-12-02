use std::{thread, time::Duration};

use intuitive::{
  component,
  components::{Fixed, Padding, Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  render::hooks::{use_effect, use_state},
  style::Color,
  terminal::Terminal,
  utils::layout::{Alignment, Amount},
};

#[component(Root)]
fn render() -> AnyElement {
  let seconds = use_state(|| 0);
  let seconds_clone = seconds.clone();

  use_effect(|| {
    thread::spawn(move || loop {
      thread::sleep(Duration::from_secs(1));

      seconds_clone.update(|seconds| seconds + 1).unwrap();
    });
  });

  render! {
    Padding(amount: Amount::Percentage(10)) {
      Fixed(height: Amount::Fixed(3)) {
        Section(title: "Seconds", border: Color::Red) {
          Text(text: format!("This program has run for {} seconds", seconds), alignment: Alignment::Center)
        }
      }
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
