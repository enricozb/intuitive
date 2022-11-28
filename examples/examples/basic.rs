use intuitive::{
  components::{Component, Text},
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
      Text(text: "Hello, world!")
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
