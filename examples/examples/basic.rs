use intuitive::{
  component,
  components::{Section, Text},
  element::Any as AnyElement,
  error::Result,
  render,
  terminal::Terminal,
};

#[component(Root)]
fn render() -> AnyElement {
  render! {
    Section(title: "Hello, world!") {
      Text(text: "Here's a basic example of intuitive!")
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
