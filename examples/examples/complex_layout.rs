use intuitive::{
  component,
  components::{Section, Stack},
  element::Any as AnyElement,
  error::Result,
  render,
  terminal::Terminal,
  utils::layout::{Flex::*, FlexDirection},
};

#[component(Root)]
fn render() -> AnyElement {
  render! {
    Stack(direction: FlexDirection::Row, flex: [Grow(1), Grow(2)]) {
      Commits()
      Files()
    }
  }
}

#[component(Commits)]
fn render() -> AnyElement {
  render! {
    Stack(direction: FlexDirection::Column, flex: [Fixed(3), Grow(1), Grow(1), Grow(1), Fixed(3)]) {
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
    Stack(direction: FlexDirection::Column, flex: [Grow(3), Grow(1)]) {
      Section(title: "Unstaged Changes")
      Section(title: "Command Log")
    }
  }
}

fn main() -> Result<()> {
  Terminal::new()?.render(&render! { Root() })?;

  Ok(())
}
