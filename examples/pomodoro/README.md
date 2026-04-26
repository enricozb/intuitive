# Pomodoro example

A two-pane pomodoro timer demonstrating the `intuitive` runtime.

```
┌── Tasks ──────┐ ┌── Timer ──────┐
│ [ ] Task 1    │ │               │
│ [x] Task 2    │ │     25:00     │
│ [ ] Task 3    │ │     paused    │
│               │ │               │
└───────────────┘ └───────────────┘
```

## Run

```sh
cargo run -p pomodoro
```

## Keys

| Key       | Action                                      |
|-----------|---------------------------------------------|
| `Tab`     | Toggle focus between Tasks and Timer        |
| `↑`/`↓`   | Move task selection (when Tasks is focused) |
| `Space`   | Toggle current task (Tasks) or start/stop the timer (Timer) |
| `Esc`     | Quit                                        |
| `Ctrl-C`  | Quit                                        |
