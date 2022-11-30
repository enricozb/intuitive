use std::{thread, time::Duration};

use intuitive::{
  components::{Component, Padding, Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  render::hooks::{use_effect, use_state},
  terminal::Terminal,
  utils::layout::Amount,
};

#[derive(Clone, Default)]
pub struct Root {}

impl Component for Root {
  fn render(&self) -> AnyElement {
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
        Section(title: "Seconds") {
          Text(text: format!("This program has run for {} seconds", seconds))
        }
      }
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
