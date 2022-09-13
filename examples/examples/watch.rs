use std::{
  process::{Command, Output},
  thread,
  time::Duration,
};

use chrono::Local;
use clap::Parser;
use intuitive::{
  component,
  components::{stack::Flex::*, HStack, Section, Text, VStack},
  error::Result,
  on_key, render,
  state::State,
  style::Color,
  terminal::Terminal,
  text::Span,
};

#[component(Top)]
fn render(interval: u64, command: String) {
  let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

  render! {
    HStack(flex: [Block(10), Grow(1), Block(21)]) {
      Section(title: "Every", border: Color::DarkGray) {
        Text(text: format!("{}s", interval))
      }
      Section(title: "Command", border: Color::DarkGray) {
        Text(text: command)
      }
      Section(title: "Time", border: Color::DarkGray) {
        Text(text: date)
      }
    }
  }
}

#[component(CommandOutput)]
fn render(output: State<Option<Output>>) {
  let text = output.inspect(|output| match output {
    Some(output) => {
      if output.status.success() {
        Span::new(String::from_utf8_lossy(&output.stdout), Color::White)
      } else {
        Span::new(String::from_utf8_lossy(&output.stderr), Color::Red)
      }
    }

    None => Span::new(".. waiting for command to finish ..", Color::Gray),
  });

  render! {
    Text(text)
  }
}

#[component(Root)]
fn render(args: Args, output: State<Option<Output>>) {
  let on_key = on_key! {
    KeyEvent { code: Char('q'), .. } => event::quit()
  };

  render! {
    VStack(flex: [Block(3), Grow(1)], on_key) {
      Top(interval: args.interval, command: args.command.clone())
      CommandOutput(output: output.clone())
    }
  }
}

#[derive(Parser, Debug, Default, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(short = 'n', long)]
  interval: u64,

  #[clap(value_parser)]
  command: String,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let output = State::<Option<Output>>::default();

  {
    let output = output.clone();
    let args = args.clone();

    thread::spawn(move || {
      let mut cmd = Command::new("sh");
      let cmd = cmd.args(["-c".to_string(), args.command]);

      let interval = Duration::from_secs(args.interval);

      loop {
        output.set(Some(cmd.output().expect("cmd::output panicked")));

        thread::sleep(interval);
      }
    });
  }

  Terminal::new(Root::new(args, output))?.run()
}
