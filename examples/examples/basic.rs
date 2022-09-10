use intuitive::{
  component,
  components::{Centered, Section, Text},
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
    Centered(on_key) {
      Section(title: "Input Box") {
        Text(text: text.get())
      }
    }
  }
}

fn main() -> Result<()> {
  Terminal::new(Root::new())?.run()
}
