//! Modal example: demonstrates that `ui!{}` produces standalone
//! `Element` values that can be bound to locals, passed around as
//! props, and composed by other components.

use intuitive::{
  buffer::{
    Color::{Green, Red, Yellow},
    Decoration::Bold,
    Style,
  },
  components::{Border, Centered, HStack, Text, VStack},
  geometry::{
    Align::Center,
    Flex::{Cell, Frac},
  },
  macros::{component, ui},
  runtime::{Context, Element, Runtime},
  signal::Signal,
};
use tokio::runtime::Builder;

#[component]
fn Root(ctx: &mut Context<'_>) -> Element {
  let clicks = ctx.new_signal(0);
  let clicks_now = clicks.get(ctx);
  ui! {
    VStack(.flex: [Cell(3), Frac(1)]) {
      Border(.title: "Usage", .style: Red) {
        Text(text: "Click the clicker to increment, Esc to quit", .align: Center)
      }
      HStack {
        Centered(.height: Cell(3), .width: Frac(2)) {
            Clicker(clicks, .style: Bold)
        }
        Border(.title: "Counter", .style: Yellow) {
          Text(text: format!("{clicks_now} clicks"), .align: Center)
        }
      }
    }
  }
}

#[component]
fn Clicker(ctx: &mut Context<'_>, clicks: Signal<i32>, #[default] style: Style) -> Element {
  ctx.on_click({
    let clicks = clicks.clone();
    move |_| clicks.update(|c| *c += 1)
  });
  ui! {
    Border(.style: Green) {
      Text(text: "Click me!", .style, .align: Center)
    }
  }
}

fn main() -> std::io::Result<()> {
  let rt = Builder::new_multi_thread().enable_all().build()?;
  rt.block_on(Runtime::render_root(ui! { Root() }))
}
