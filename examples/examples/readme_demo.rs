use intuitive::{
  component,
  components::{stack::Flex::*, HStack, Section, Text, VStack},
  error::Result,
  on_key, render,
  state::use_state,
  terminal::Terminal,
};

#[component(Root)]
fn render() {
  let text = use_state(|| String::new());

  let on_key = on_key! { [text]
    KeyEvent { code: Char(c), .. } => text.mutate(|text| text.push(c)),
    KeyEvent { code: Backspace, .. } => text.mutate(|text| text.pop()),
    KeyEvent { code: Esc, .. } => event::quit(),
  };

  render! {
    VStack(flex: [Block(3), Grow(1)], on_key) {
      Section(title: "Input") {
        Text(text: text.get())
      }

      HStack(flex: [1, 2, 3]) {
        Section(title: "Column 1")
        Section(title: "Column 2")
        Section(title: "Column 3")
      }
    }
  }
}

fn main() -> Result<()> {
  Terminal::new(Root::new())?.run()
}
