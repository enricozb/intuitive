//! Pomodoro example: task list on the left, countdown timer on the right.
//!
//! Bindings:
//!
//! - Tab — toggle focus between panes.
//! - `n` — open the centered modal input to *add* a new task.
//! - `e` — (with the Tasks pane focused) open the same modal pre-filled with
//!   the currently-selected task's text to *edit* it.
//! - Enter (inside the modal) — submit. Whitespace-only text dismisses without
//!   changes (this is also the "cancel" path).
//! - Space — toggle the selected task's check / start-stop the timer.
//! - Ctrl-C / Esc — quit.
//!
//! Note: the runtime treats Esc as "quit", so we don't use it to cancel
//! the modal; submitting an empty input is the cancel path.
//!
//! Add and edit share a single "task editor" modal: the only thing that
//! differs is what the submit action does. We model that as an
//! `EditMode` signal (`New` vs `Edit(idx)`) read at submit time, plus
//! the existing `editing` flag / `editor_text` buffer that already
//! drove the add-only version.

use std::{fmt::Display, string::ToString, time::Duration};

use intuitive::{
  buffer::{Color, Style},
  components::{Border, Centered, Empty, HStack, Input, List, Modal, Text, VStack},
  event::{KeyCode, KeyEventKind},
  geometry::{
    Align,
    Flex::{Cell, Frac},
  },
  macros::{component, ui},
  runtime::{Context, Element, Runtime},
  signal::Signal,
};

const POMODORO_SECS: u32 = 25 * 60;
const BORDER_FOCUSED_STYLE: Style = Style::new().fg(Color::Green).bold(true);
const BORDER_DEFAULT_STYLE: Style = Style::new().fg(Color::DarkGrey);

#[derive(Clone)]
struct Task {
  text: String,
  checked: bool,
}

impl Task {
  fn new(text: impl Into<String>) -> Self {
    Self {
      text: text.into(),
      checked: false,
    }
  }
}

impl Display for Task {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let check = if self.checked { "x" } else { " " };
    write!(f, "[{check}] {}", self.text)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
  Tasks,
  Timer,
}

impl Pane {
  fn toggle(&mut self) {
    match self {
      Self::Tasks => *self = Self::Timer,
      Self::Timer => *self = Self::Tasks,
    }
  }
}

/// Which task-editor flow the modal is currently driving. Read at
/// submit time to decide whether the typed text turns into a new task
/// (`New`) or replaces the text of an existing one (`Edit(idx)`).
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum EditMode {
  #[default]
  New,
  Edit(usize),
}

/// Root component: owns every signal, registers the global key
/// handlers and the timer-tick task, and lays out the two panes side
/// by side under a `Modal` that hosts a single shared task editor.
#[component]
fn Root(ctx: &mut Context<'_>) -> Element {
  let focus = ctx.new_signal(Pane::Tasks);
  let editing = ctx.new_signal(false);
  let editor_mode = ctx.new_signal(EditMode::New);
  let editor_text = ctx.new_signal(String::new());

  let remaining = ctx.new_signal(POMODORO_SECS);
  let running = ctx.new_signal(false);

  let tasks = ctx.new_signal_with(|| vec![Task::new("Press n to create a task!")]);
  let selected = ctx.new_signal(0);

  let focus_now = focus.get(ctx);
  let modal_open = editing.get(ctx);

  ctx.on_key({
    let editing = editing.clone();
    let editor_mode = editor_mode.clone();
    let editor_text = editor_text.clone();
    let tasks = tasks.clone();
    let selected = selected.clone();
    let focus = focus.clone();
    move |k| {
      if k.kind != KeyEventKind::Press {
        return;
      }
      // Read the current visibility on each keystroke (the closure is
      // re-registered every render but `editing.peek()` is the
      // cheapest way to branch without re-subscribing).
      let is_open = editing.peek();
      if is_open {
        if k.code == KeyCode::Enter {
          // Enter submits. Whitespace-only text is treated as "cancel":
          // we close the modal without changing anything. Otherwise we
          // dispatch on `editor_mode`:
          //   - `New`        → push a new task.
          //   - `Edit(idx)`  → replace the text of the task at `idx`.
          let text = editor_text.peek().trim().to_string();
          if !text.is_empty() {
            match editor_mode.peek() {
              EditMode::New => tasks.update(|ts| ts.push(Task::new(text))),
              EditMode::Edit(idx) => tasks.update(|ts| {
                if let Some(task) = ts.get_mut(idx) {
                  task.text = text;
                }
              }),
            }
          }
          editor_text.set(String::new());
          editor_mode.set(EditMode::New);
          editing.set(false);
        }
      } else {
        match k.code {
          KeyCode::Tab => focus.update(Pane::toggle),
          KeyCode::Char('n') => {
            // "New": start with an empty buffer.
            editor_text.set(String::new());
            editor_mode.set(EditMode::New);
            editing.set(true);
          }
          KeyCode::Char('e') if focus.peek() == Pane::Tasks => {
            // "Edit": only meaningful with the Tasks pane focused
            // *and* a real task selected. Pre-fill the input buffer
            // with the current task text so the user can amend it
            // rather than retype from scratch.
            let idx = selected.peek();
            let current = tasks.peek().get(idx).map(|t| t.text.clone());
            if let Some(text) = current {
              editor_text.set(text);
              editor_mode.set(EditMode::Edit(idx));
              editing.set(true);
            }
          }
          _ => {}
        }
      }
    }
  });

  ctx.spawn({
    let remaining = remaining.clone();
    let running = running.clone();
    async move {
      let mut interval = tokio::time::interval(Duration::from_secs(1));
      loop {
        interval.tick().await;
        if running.peek() && remaining.peek() > 0 {
          remaining.update(|r| *r = r.saturating_sub(1));
        }
      }
    }
  });

  // The modal's title reflects which flow is active so the user sees
  // "New Task" vs "Edit Task" depending on how they opened it. We
  // peek (not get) because subscribing here would re-render Root just
  // to flip the title — but Root already re-renders when `editing`
  // flips, which is the only moment `editor_mode` matters.
  let modal_title = match editor_mode.peek() {
    EditMode::New => "New Task",
    EditMode::Edit(_) => "Edit Task",
  };

  ui! {
    Modal(visible: editing.clone()) {
      HStack(.flex: [Frac(1), Frac(1)]) {
        TasksPane(
          focused: focus_now == Pane::Tasks,
          modal_open: modal_open,
          selected: selected,
          tasks: tasks,
        )
        TimerPane(
          focused: focus_now == Pane::Timer,
          modal_open: modal_open,
          remaining: remaining,
          running: running,
        )
      }
      Centered(.pad_x: Frac(1), .width: Frac(1), .height: Cell(3)) {
        Border(.title: modal_title.to_string(), .style: BORDER_FOCUSED_STYLE) {
          Input(value: editor_text, focused: editing)
        }
      }
    }
  }
}

/// Left pane: bordered "Tasks" wrapping the task-list body. `modal_open`
/// is threaded through so the body can suppress its key bindings while
/// the task-editor modal owns the keyboard. `selected` is owned by
/// `Root` (so the `e` key can read it) and passed in as a prop.
#[component]
pub fn TasksPane(focused: bool, modal_open: bool, selected: Signal<usize>, tasks: Signal<Vec<Task>>) -> Element {
  let style = if focused {
    BORDER_FOCUSED_STYLE
  } else {
    BORDER_DEFAULT_STYLE
  };

  ui! {
    Border(.title: "Tasks", .style: style) {
      TasksBody(
        focused: focused,
        modal_open: modal_open,
        selected: selected,
        tasks: tasks.clone(),
      )
    }
  }
}

/// Body of the tasks pane: feeds the formatted task lines into a `List`
/// and wires up the key bindings.
#[component]
pub fn TasksBody(
  ctx: &mut Context<'_>,
  focused: bool,
  modal_open: bool,
  selected: Signal<usize>,
  tasks: Signal<Vec<Task>>,
) -> Element {
  let sel = selected.get(ctx);
  let len = tasks.map(ctx, Vec::len);

  if focused && !modal_open {
    ctx.on_key({
      let selected = selected.clone();
      let tasks = tasks.clone();
      move |k| {
        if k.kind != KeyEventKind::Press {
          return;
        }
        match k.code {
          KeyCode::Up | KeyCode::Char('k') => selected.update(|s| *s = s.saturating_sub(1)),
          KeyCode::Down | KeyCode::Char('j') => selected.update(|s| *s = (*s + 1).min(len - 1)),
          KeyCode::Char(' ') => {
            tasks.update(move |tasks| {
              if let Some(task) = tasks.get_mut(sel) {
                task.checked = !task.checked;
              }
            });
          }
          _ => {}
        }
      }
    });
  }

  let items: Vec<String> = tasks.map(ctx, |tasks| tasks.iter().map(ToString::to_string).collect());

  let selected_style = if focused {
    Style::new().fg(Color::Yellow).bold(true)
  } else {
    Style::new().bold(true)
  };

  ui! {
    List(items: items, .selected: sel, .selected_style: selected_style)
  }
}

/// Right pane: bordered "Timer" wrapping the countdown body. `modal_open`
/// is threaded through so the body can suppress its key bindings while
/// the "new task" modal owns the keyboard.
#[component]
pub fn TimerPane(focused: bool, modal_open: bool, remaining: Signal<u32>, running: Signal<bool>) -> Element {
  let style = if focused {
    BORDER_FOCUSED_STYLE
  } else {
    BORDER_DEFAULT_STYLE
  };

  ui! {
    Border(.title: "Timer", .style: style) {
      TimerBody(focused: focused, modal_open: modal_open, remaining: remaining, running: running)
    }
  }
}

/// Body of the timer pane: draws the countdown via `Text` + `VStack`,
/// and handles space-to-toggle.
#[component]
pub fn TimerBody(
  ctx: &mut Context<'_>,
  focused: bool,
  modal_open: bool,
  remaining: Signal<u32>,
  running: Signal<bool>,
) -> Element {
  let secs = remaining.get(ctx);
  let is_running = running.get(ctx);

  if focused && !modal_open {
    ctx.on_key({
      let running = running.clone();
      move |k| {
        if k.kind == KeyEventKind::Press && k.code == KeyCode::Char(' ') {
          running.update(|r| *r = !*r);
        }
      }
    });
  }

  let mins = secs / 60;
  let s = secs % 60;
  let time = format!("{mins:02}:{s:02}");
  let status = if is_running { "running" } else { "paused" };
  let time_color = if secs == 0 {
    Color::Red
  } else if is_running {
    Color::Green
  } else {
    Color::Yellow
  };
  let time_style = Style::new().fg(time_color);

  // Layout: a Frac region above hosting the time number anchored to
  // its bottom (so it lands at the visual middle of the upper
  // half), one Cell row each for status and the hint, with a
  // Frac(1) spacer in between.
  ui! {
    VStack(.flex: [Frac(1), Cell(1), Frac(1), Cell(1), Cell(1)]) {
      Text(text: time, .style: time_style, .align: Align::S)
      Text(text: status, .style: Color::DarkGrey, .align: Align::N)
      Empty()
      Text(text: "Space to start/stop", .style: Color::DarkGrey, .align: Align::S)
      Empty()
    }
  }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  Runtime::render_root(ui! { Root() }).await
}
