use intuitive::{
  components::{Component, Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  terminal::Terminal,
};

#[derive(Clone, Default)]
pub struct Root {}

impl Component for Root {
  fn render(&self) -> AnyElement {
    render! {
      Section(title: "Hello, world!") {
        Text(text: "Here's a basic example of intuitive!")
      }
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
