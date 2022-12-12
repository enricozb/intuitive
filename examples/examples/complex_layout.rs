use intuitive::{
  component,
  components::{HStack, Section, VStack},
  element::Any as AnyElement,
  error::Result,
  render,
  terminal::Terminal,
  utils::layout::Flex::*,
};

#[component(Root)]
fn render() -> AnyElement {
  render! {
    HStack(flex: [Grow(1), Grow(2)]) {
      Commits()
      Files()
    }
  }
}

#[component(Commits)]
fn render() -> AnyElement {
  render! {
    VStack(flex: [Fixed(3), Grow(1), Grow(1), Grow(1), Fixed(3)]) {
      Section(title: "Status")
      Section(title: "Files")
      Section(title: "Branches")
      Section(title: "Commits")
      Section(title: "Stash")
    }
  }
}

#[component(Files)]
fn render() -> AnyElement {
  render! {
    VStack(flex: [Grow(3), Grow(1)]) {
      Section(title: "Unstaged Changes")
      Section(title: "Command Log")
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(Root {})?;

  Ok(())
}
