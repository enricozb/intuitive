use intuitive::{
  component,
  components::{Padding, Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  terminal::Terminal,
  utils::layout::Amount,
};

#[component(TextBox)]
fn render(title: String, text: String) -> AnyElement {
  render! {
    Section(title) {
      Text(text)
    }
  }
}

#[component(Root)]
fn render() -> AnyElement {
  render! {
    Padding(amount: Amount::Percentage(10)) {
      TextBox(title: "Hello, world!", text: "I'm inside a text box.")
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(Root {})?;

  Ok(())
}
