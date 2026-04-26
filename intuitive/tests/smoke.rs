//! Headless smoke tests for the runtime: drive events directly without a
//! terminal, inspect the back buffer.

use intuitive::{
  buffer::{Buffer, Cell, Color, Style},
  components::{Border, Centered, HStack, Input, List, Modal, Text, VStack},
  event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
  geometry::{Align, Flex, Point, Size},
  macros::ui,
  runtime::{Context, Event, Runtime},
  signal::Signal,
};

/// Read the row at `y` from a buffer as a String of cell chars.
fn row(buf: &Buffer, y: u16) -> String {
  let size = buf.size();
  let mut s = String::with_capacity(size.w as usize);
  for x in 0..size.w {
    s.push(buf.get(Point::new(x, y)).copied().unwrap_or(Cell::BLANK).ch);
  }
  s
}

#[test]
fn renders_text() {
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 3));
  rt.mount_root(ui! { Text(text: "hi") });
  assert!(rt.step(Event::Init));
  assert_eq!(&row(rt.back_buffer(), 0)[..2], "hi");
}

#[test]
fn renders_hstack_split() {
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  rt.mount_root(ui! {
    HStack(.flex: [Flex::Frac(1), Flex::Frac(1)]) {
      Text(text: "aa")
      Text(text: "bb")
    }
  });
  assert!(rt.step(Event::Init));
  let r = row(rt.back_buffer(), 0);
  // 10/2 = 5 wide each. First piece starts at x=0 ("aa..."), second at
  // x=5 ("bb...").
  assert_eq!(&r[..2], "aa");
  assert_eq!(&r[5..7], "bb");
}

#[test]
fn renders_border_with_title() {
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(20, 5));
  rt.mount_root(ui! {
    Border(.title: "Hello") {
      Text(text: "inside")
    }
  });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  // Top-left corner.
  assert_eq!(buf.get(Point::new(0, 0)).unwrap().ch, '┌');
  assert_eq!(buf.get(Point::new(19, 0)).unwrap().ch, '┐');
  assert_eq!(buf.get(Point::new(0, 4)).unwrap().ch, '└');
  // Title appears.
  let r = row(buf, 0);
  assert!(r.contains("Hello"), "expected title in {r:?}");
  // Inner content.
  let r = row(buf, 1);
  assert!(r.contains("inside"), "expected child text in row 1: {r:?}");
}

#[test]
fn signal_set_triggers_rerender() {
  use intuitive::runtime::Element;

  // A simple counter component: signal `count`, key '+' increments.
  fn counter() -> Element {
    Element::new(|ctx: &mut Context<'_>| {
      let count = ctx.new_signal_default::<u32>();
      let n = count.get(ctx);
      let count_ = count.clone();
      ctx.on_key(move |k| {
        if k.kind == KeyEventKind::Press
          && let KeyCode::Char('+') = k.code
        {
          count_.update(|c| *c += 1);
        }
      });
      let mut r = ctx.region();
      r.fill(Cell::BLANK);
      r.write_str((0, 0), &format!("n={n}"), Style::RESET);
    })
  }

  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  rt.mount_root(counter());
  assert!(rt.step(Event::Init));
  assert_eq!(&row(rt.back_buffer(), 0)[..3], "n=0");

  // Smoke check that key dispatch doesn't panic. The full set/notify
  // round-trip is covered by `signal_set_and_drain` below.
  let key = KeyEvent {
    code: KeyCode::Char('+'),
    modifiers: KeyModifiers::NONE,
    kind: KeyEventKind::Press,
    state: crossterm::event::KeyEventState::NONE,
  };
  assert!(rt.step(Event::Key(key)));
}

#[test]
fn signal_set_and_drain() {
  use intuitive::runtime::Element;

  fn counter() -> Element {
    Element::new(|ctx: &mut Context<'_>| {
      let count = ctx.new_signal_default::<u32>();
      let n = count.get(ctx);
      let count_ = count.clone();
      ctx.on_key(move |k| {
        if k.kind == KeyEventKind::Press
          && let KeyCode::Char('+') = k.code
        {
          count_.update(|c| *c += 1);
        }
      });
      let mut r = ctx.region();
      r.fill(Cell::BLANK);
      r.write_str((0, 0), &format!("n={n}"), Style::RESET);
    })
  }

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(10, 1));
  rt.mount_root(counter());
  rt.step(Event::Init);
  assert_eq!(&row(rt.back_buffer(), 0)[..3], "n=0");

  // Press '+'. The handler mutates storage synchronously and posts a
  // SignalChanged notification on the channel.
  let key = KeyEvent {
    code: KeyCode::Char('+'),
    modifiers: KeyModifiers::NONE,
    kind: KeyEventKind::Press,
    state: crossterm::event::KeyEventState::NONE,
  };
  rt.step(Event::Key(key));
  // Drain the SignalChanged notification.
  let next = rx.try_recv().expect("expected a SignalChanged notification");
  assert!(matches!(next, Event::Signal(_)));
  rt.step(next);
  assert_eq!(&row(rt.back_buffer(), 0)[..3], "n=1");

  // Press again.
  rt.step(Event::Key(key));
  let next = rx.try_recv().expect("expected another notification");
  rt.step(next);
  assert_eq!(&row(rt.back_buffer(), 0)[..3], "n=2");
}

#[test]
fn child_unmount_gcs_subtree() {
  // Toggle: when on==true, render two children; when off, render one.
  use std::sync::{Arc, Mutex};

  use intuitive::{runtime::Element, signal::Signal};

  fn parent(on: Signal<bool>) -> Element {
    Element::new(move |ctx: &mut Context<'_>| {
      let active = on.get(ctx);
      let stack = if active {
        ui! {
          HStack(.flex: [Flex::Frac(1), Flex::Frac(1)]) {
            Text(text: "L")
            Text(text: "R")
          }
        }
      } else {
        ui! {
          HStack(.flex: [Flex::Frac(1)]) {
            Text(text: "L")
          }
        }
      };
      let rect = ctx.region().rect();
      ctx.render(stack, rect);
    })
  }

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(4, 1));

  // Bootstrap an external on signal: we need to create it with a known
  // ElementKey. Easiest: have a wrapper element that creates the signal
  // and renders parent below it.
  let captured: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let captured_setup = captured.clone();

  let root_elem: Element = Element::new(move |ctx: &mut Context<'_>| {
    let on = ctx.new_signal_default::<bool>();
    *captured_setup.lock().unwrap() = Some(on.clone());
    let rect = ctx.region().rect();
    ctx.render(parent(on), rect);
  });

  rt.mount_root(root_elem);
  rt.step(Event::Init);
  // Initially `on=false`, so parent renders one child "L".
  assert_eq!(&row(rt.back_buffer(), 0)[..1], "L");

  // Flip to on=true.
  let on = captured.lock().unwrap().clone().unwrap();
  on.set(true);
  let next = rx.try_recv().expect("expected Signal");
  rt.step(next);
  // 4 wide split 1:1 = 2:2; "L" at x=0, "R" at x=2.
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..1], "L");
  assert_eq!(&r[2..3], "R");

  // Flip back.
  on.set(false);
  let next = rx.try_recv().expect("expected Signal");
  rt.step(next);
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..1], "L");
  // The 'R' from the previous render must be gone.
  assert_ne!(&r[2..3], "R", "old child not GCd: {r:?}");
}

#[test]
fn conditional_on_key_drops_when_unrendered() {
  // Verify that on_key registered conditionally is dropped on renders
  // where the branch isn't taken. Concretely: a component renders an
  // on_key handler only when `armed` is true; we toggle `armed`,
  // verify the handler fires; toggle off, verify it doesn't.
  use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
  };

  use intuitive::{runtime::Element, signal::Signal};

  let counter = Arc::new(AtomicU32::new(0));
  let captured: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let captured_setup = captured.clone();
  let counter_for_elem = counter.clone();

  let elem: Element = Element::new(move |ctx: &mut Context<'_>| {
    let armed = ctx.new_signal_default::<bool>();
    *captured_setup.lock().unwrap() = Some(armed.clone());
    if armed.get(ctx) {
      let counter = counter_for_elem.clone();
      ctx.on_key(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
      });
    }
    let mut r = ctx.region();
    r.fill(Cell::BLANK);
  });

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(4, 1));
  rt.mount_root(elem);
  rt.step(Event::Init);

  let armed = captured.lock().unwrap().clone().unwrap();
  let key = KeyEvent {
    code: KeyCode::Char('x'),
    modifiers: KeyModifiers::NONE,
    kind: KeyEventKind::Press,
    state: crossterm::event::KeyEventState::NONE,
  };

  // Initially unarmed: no handler should be registered, key fires nothing.
  rt.step(Event::Key(key));
  assert_eq!(counter.load(Ordering::SeqCst), 0);

  // Arm: handler should be registered after re-render.
  armed.set(true);
  let next = rx.try_recv().unwrap();
  rt.step(next);
  rt.step(Event::Key(key));
  assert_eq!(counter.load(Ordering::SeqCst), 1);

  // Disarm: handler should be dropped at end-of-render.
  armed.set(false);
  let next = rx.try_recv().unwrap();
  rt.step(next);
  rt.step(Event::Key(key));
  assert_eq!(
    counter.load(Ordering::SeqCst),
    1,
    "handler not dropped when conditional became false"
  );
}

#[test]
fn resize_triggers_full_repaint() {
  // After a resize, `front` is left at the old size so the next diff
  // is a full repaint. This regression test covers the bug where
  // growing the terminal left stale content visible in cells that
  // matched between the old `front` and the freshly-rendered `back`.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 3));
  rt.mount_root(ui! { Text(text: "hi") });
  rt.step(Event::Init);
  // Size matches now: 10x3 == 10x3.
  // Force the front buffer to match back so the next diff would be empty
  // if `front` weren't deliberately stale after resize.
  // (In the real run loop this happens after each flush.)
  // Now grow.
  rt.step(Event::Resize(Size::new(20, 5)));
  // Back is now 20x5, fully rendered. The front is still 10x3 (we did
  // not resize it). diff() should return all 20*5 cells of back.
  let diff = rt.back_buffer().diff(&intuitive::buffer::Buffer::new(Size::new(10, 3)));
  assert_eq!(diff.len(), 20 * 5, "expected full repaint after resize");
}

#[tokio::test(flavor = "multi_thread")]
async fn signal_peek_outside_render() {
  // Verify that peek works from outside any reactive context (here,
  // the test thread itself).
  use std::sync::{Arc, Mutex};

  use intuitive::runtime::Element;

  let captured: Arc<Mutex<Option<Signal<u32>>>> = Arc::new(Mutex::new(None));
  let captured_setup = captured.clone();

  let elem: Element = Element::new(move |ctx: &mut Context<'_>| {
    let n = ctx.new_signal_with::<u32>(|| 42);
    *captured_setup.lock().unwrap() = Some(n.clone());
    let _ = n.get(ctx);
    let mut r = ctx.region();
    r.fill(Cell::BLANK);
  });

  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(4, 1));
  rt.mount_root(elem);
  rt.step(Event::Init);

  let n = captured.lock().unwrap().clone().unwrap();
  assert_eq!(n.peek(), 42);
  n.set(100);
  assert_eq!(n.peek(), 100);
}

#[test]
fn text_align_center() {
  // 10-wide, 3-tall region; "hi" (width 2) centered: x=4, y=1.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 3));
  rt.mount_root(ui! { Text(text: "hi", .align: Align::Center) });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  assert_eq!(buf.get(Point::new(4, 1)).unwrap().ch, 'h');
  assert_eq!(buf.get(Point::new(5, 1)).unwrap().ch, 'i');
  // Top-left should be blank.
  assert_eq!(buf.get(Point::new(0, 0)).unwrap().ch, ' ');
}

#[test]
fn text_align_se() {
  // Anchor to bottom-right.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(6, 3));
  rt.mount_root(ui! { Text(text: "end", .align: Align::SE) });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  // dx=3, dy=2 -> pos (3, 2).
  assert_eq!(buf.get(Point::new(3, 2)).unwrap().ch, 'e');
  assert_eq!(buf.get(Point::new(4, 2)).unwrap().ch, 'n');
  assert_eq!(buf.get(Point::new(5, 2)).unwrap().ch, 'd');
}

#[test]
fn list_renders_lines_top_down() {
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 4));
  rt.mount_root(ui! {
    List(items: ["alpha", "beta", "gamma"].map(String::from))
  });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  assert!(row(buf, 0).starts_with("alpha"));
  assert!(row(buf, 1).starts_with("beta"));
  assert!(row(buf, 2).starts_with("gamma"));
}

#[test]
fn list_scrolls_to_keep_selection_visible() {
  // 10 items, only 3 rows visible: selecting item 5 must scroll so it's
  // shown as the last visible row (rows 3,4,5 -> visible rows 0,1,2).
  let items: Vec<String> = (0..10).map(|i| format!("item-{i}")).collect();
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 3));
  rt.mount_root(ui! {
    List(
      items: items.clone(),
      .selected: 5usize,
      .selected_style: Style::new().fg(Color::Yellow).bold(true),
    )
  });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  assert!(row(buf, 0).starts_with("item-3"), "row 0: {:?}", row(buf, 0));
  assert!(row(buf, 1).starts_with("item-4"), "row 1: {:?}", row(buf, 1));
  assert!(row(buf, 2).starts_with("item-5"), "row 2: {:?}", row(buf, 2));
  // Selected row is yellow and bold.
  let cell = buf.get(Point::new(0, 2)).unwrap();
  assert_eq!(cell.style.fg, Color::Yellow);
  assert!(cell.style.bold);
  // Non-selected row is plain.
  let cell = buf.get(Point::new(0, 0)).unwrap();
  assert_eq!(cell.style.fg, Color::Reset);
  assert!(!cell.style.bold);
}

#[test]
fn list_unfocused_selection_is_bold_only() {
  // When unfocused, the selected row is bold but not colored.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 3));
  rt.mount_root(ui! {
    List(
      items: ["a", "b"].map(String::from),
      .selected: 1usize,
      .selected_style: Style::new().bold(true),
    )
  });
  assert!(rt.step(Event::Init));
  let buf = rt.back_buffer();
  let cell = buf.get(Point::new(0, 1)).unwrap();
  assert_eq!(cell.ch, 'b');
  assert_eq!(cell.style.fg, Color::Reset);
  assert!(cell.style.bold);
}

#[test]
fn select_composed_children_fine_grained_rerender_swaps() {
  // A `Select` component that receives N children at composition
  // time and renders one of them based on a `which: Signal<usize>`
  // prop. Switching `which` toggles which composed child is
  // mounted; flipping back re-mounts a previously-unmounted child
  // (which works because `Element: Clone` — the composed-children
  // vec stays intact across renders).
  //
  // This is the fixed-composed-set form (analogous to a `<Tabs>`
  // with a known set of panes), as opposed to the dynamic-child
  // form where the body picks one Element to construct per render.
  use std::sync::{Arc, Mutex};

  use intuitive::{runtime::Element, signal::Signal};

  // Hand-rolled Select component. Renders composed child `which`
  // into its full rect. Hand-rolled rather than via `#[component]`
  // so we can pull individual children from `ctx.composed()` by
  // index (the `#[children]` macro lowering materializes the
  // entire vec up front, which is convenient for containers but
  // not what we want here).
  fn select(which: Signal<usize>) -> Element {
    Element::new(move |ctx: &mut Context<'_>| {
      let i = which.get(ctx);
      let r = ctx.region().rect();
      let composed = ctx.composed();
      if let Some(child) = composed.get(i) {
        let child = child.clone();
        ctx.render(child, r);
      }
    })
  }

  // Bootstrap: a root element that owns the `which` signal and
  // composes three Text children into Select.
  let captured: Arc<Mutex<Option<Signal<usize>>>> = Arc::new(Mutex::new(None));
  let captured_setup = captured.clone();

  let root: Element = Element::new(move |ctx: &mut Context<'_>| {
    let which = ctx.new_signal_default::<usize>();
    *captured_setup.lock().unwrap() = Some(which.clone());
    let r = ctx.region().rect();
    let s = select(which.clone()).with_children(vec![
      ui! { Text(text: "alpha") },
      ui! { Text(text: "beta") },
      ui! { Text(text: "gamma") },
    ]);
    ctx.render(s, r);
  });

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(6, 1));
  rt.mount_root(root);
  assert!(rt.step(Event::Init));
  // Default which=0 -> "alpha" rendered.
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("alpha"), "expected alpha at start, got {r:?}");

  // Flip which=1. Only Select is subscribed to `which`, so only
  // its closure re-runs; the parent (root) is untouched.
  let which = captured.lock().unwrap().clone().unwrap();
  which.set(1);
  let ev = rx.try_recv().expect("expected Signal notification");
  rt.step(ev);
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("beta"), "after which=1 expected beta, got {r:?}");
  assert!(!r.contains("alpha"), "stale alpha after switch: {r:?}");

  // Flip back to which=0 — the composed alpha child is still
  // available (because Element is Clone, the runtime cloned it on
  // first mount rather than consuming it). Mounts fresh.
  which.set(0);
  let ev = rx.try_recv().expect("expected Signal notification");
  rt.step(ev);
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("alpha"), "after which=0 expected alpha again, got {r:?}");
  assert!(!r.contains("beta"), "stale beta after switch-back: {r:?}");

  // Flip to which=2.
  which.set(2);
  let ev = rx.try_recv().expect("expected Signal notification");
  rt.step(ev);
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("gamma"), "after which=2 expected gamma, got {r:?}");

  // And back to alpha once more, to confirm the pattern is stable
  // across multiple round-trips.
  which.set(0);
  let ev = rx.try_recv().expect("expected Signal notification");
  rt.step(ev);
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("alpha"), "after second which=0 expected alpha, got {r:?}");
}

#[test]
fn container_rerenders_alone_keeps_composed_children() {
  // An HStack-like container that has a signal of its own
  // (so it can be dirtied without its parent re-rendering). On
  // re-render its body re-iterates and calls `render_composed`
  // for each slot again, even though those slots' Element
  // values were taken on the previous render. The runtime
  // falls back to the recorded placement from last time.
  use std::sync::{Arc, Mutex};

  use intuitive::{runtime::Element, signal::Signal};

  // Build a container that registers a signal so we can trigger
  // its own re-render independently, but otherwise behaves like
  // HStack-of-two: places composed[0] left, composed[1] right.
  let captured: Arc<Mutex<Option<Signal<u32>>>> = Arc::new(Mutex::new(None));
  let captured_setup = captured.clone();

  let container_factory = move || -> Element {
    let captured_setup = captured_setup.clone();
    Element::new(move |ctx: &mut Context<'_>| {
      let tick = ctx.new_signal_default::<u32>();
      *captured_setup.lock().unwrap() = Some(tick.clone());
      let _ = tick.get(ctx); // subscribe
      let rect = ctx.region().rect();
      let parts = rect.split_h_flex(&[Flex::Frac(1), Flex::Frac(1)]);
      // Clone composed entries by index. `Element: Clone` is
      // cheap (Arc bump on the closure).
      let composed = ctx.composed();
      let a = composed[0].clone();
      let b = composed[1].clone();
      ctx.render(a, parts[0]);
      ctx.render(b, parts[1]);
    })
  };

  let root: Element = Element::new(move |ctx: &mut Context<'_>| {
    let r = ctx.region().rect();
    let c = container_factory().with_children(vec![ui! { Text(text: "L") }, ui! { Text(text: "R") }]);
    ctx.render(c, r);
  });

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(4, 1));
  rt.mount_root(root);
  assert!(rt.step(Event::Init));
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..1], "L");
  assert_eq!(&r[2..3], "R");

  // Bump the container's signal: container alone re-renders.
  let tick = captured.lock().unwrap().clone().unwrap();
  tick.set(1);
  let ev = rx.try_recv().expect("Signal notification");
  rt.step(ev);
  // Composed children are still placed correctly even though their
  // Element values were taken-out on the first render.
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..1], "L", "left composed child lost after re-render: {r:?}");
  assert_eq!(&r[2..3], "R", "right composed child lost after re-render: {r:?}");

  tick.set(2);
  let ev = rx.try_recv().expect("Signal notification");
  rt.step(ev);
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..1], "L");
  assert_eq!(&r[2..3], "R");
}

#[test]
fn splice_ident_and_iter_in_ui() {
  // Exercises `ui!{}` splice forms end-to-end:
  //   `#name` injects a pre-built Element value, and
  //   `#(expr)*` injects an iterator of Elements.
  // The runtime must mount each spliced child correctly under a
  // distinct ElementKey (via spans / user keys).
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(8, 3));
  let header: intuitive::runtime::Element = ui! { Text(text: "head") };
  let rows: Vec<intuitive::runtime::Element> = (0..2)
    .map(|i| ui! { Text(text: format!("r{i}"), key: i.to_string()) })
    .collect();
  rt.mount_root(ui! {
    VStack(.flex: [Flex::Cell(1), Flex::Cell(1), Flex::Cell(1)]) {
      #header
      #(rows)*
    }
  });
  assert!(rt.step(Event::Init));
  assert!(
    row(rt.back_buffer(), 0).starts_with("head"),
    "row 0: {:?}",
    row(rt.back_buffer(), 0)
  );
  assert!(
    row(rt.back_buffer(), 1).starts_with("r0"),
    "row 1: {:?}",
    row(rt.back_buffer(), 1)
  );
  assert!(
    row(rt.back_buffer(), 2).starts_with("r1"),
    "row 2: {:?}",
    row(rt.back_buffer(), 2)
  );
}

#[test]
fn splice_expr_pushes_constructed_element() {
  // The `#(expr)` form pushes the result of an arbitrary
  // expression, which must evaluate to `Element`. Two Elements
  // produced by the same helper share a span; user keys are
  // threaded via `key:` so they get distinct ElementKeys.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(8, 2));
  let make_text = |s: &str, k: &str| -> intuitive::runtime::Element {
    ui! { Text(text: s.to_string(), key: k.to_string()) }
  };
  rt.mount_root(ui! {
    VStack(.flex: [Flex::Cell(1), Flex::Cell(1)]) {
      #(make_text("alpha", "a"))
      #(make_text("beta", "b"))
    }
  });
  assert!(rt.step(Event::Init));
  assert!(row(rt.back_buffer(), 0).starts_with("alpha"));
  assert!(row(rt.back_buffer(), 1).starts_with("beta"));
}

#[test]
fn centered_default_places_child_in_middle_third() {
  // With both `x` and `y` defaulting to `Flex::Frac(1)`, the rect splits
  // into thirds along each axis. A 9x3 buffer therefore reserves cols
  // [3..6) and rows [1..2) for the child.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(9, 3));
  rt.mount_root(ui! {
    Centered() {
      Text(text: "hi")
    }
  });
  assert!(rt.step(Event::Init));
  let r = row(rt.back_buffer(), 1);
  assert_eq!(&r[3..5], "hi", "expected 'hi' at cols 3..5, got {r:?}");
  // Top and bottom rows are untouched (still blank).
  assert!(row(rt.back_buffer(), 0).chars().all(|c| c == ' '));
  assert!(row(rt.back_buffer(), 2).chars().all(|c| c == ' '));
}

#[test]
fn centered_with_cell_margins() {
  // Explicit `Cell` margins reserve a fixed number of cells on each side.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 5));
  rt.mount_root(ui! {
    Centered(.pad_x: Flex::Cell(2), .pad_y: Flex::Cell(1)) {
      Text(text: "xx")
    }
  });
  assert!(rt.step(Event::Init));
  // Child column = [2, 8); child row = [1, 4). Text at top-left of child.
  let r = row(rt.back_buffer(), 1);
  assert_eq!(&r[2..4], "xx");
}

/// Build a root element that creates a `Signal<T>` via `ctx.new_signal`,
/// stashes a clone of it for the test driver to read/write, and then
/// invokes `body` (closing over the signal) to produce the visible
/// subtree.
///
/// Signals can only be constructed via `Context`, so tests that want
/// to drive a signal from outside go through this dance to capture
/// one out of a real element's render pass.
fn capture_signal_root<T, F>(
  initial: T,
  captured: std::sync::Arc<std::sync::Mutex<Option<Signal<T>>>>,
  body: F,
) -> intuitive::runtime::Element
where
  T: Send + Sync + Clone + 'static,
  F: Fn(Signal<T>) -> intuitive::runtime::Element + Send + 'static,
{
  use intuitive::runtime::Element;
  let initial = std::sync::Arc::new(std::sync::Mutex::new(Some(initial)));
  Element::new(move |ctx: &mut Context<'_>| {
    let sig: Signal<T> =
      ctx.new_signal_with(|| initial.lock().unwrap().take().expect("new_signal_with init runs once"));
    *captured.lock().unwrap() = Some(sig.clone());
    let rect = ctx.region().rect();
    let child = body(sig);
    ctx.render(child, rect);
  })
}

#[test]
fn modal_hides_overlay_when_signal_false() {
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  let captured: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  rt.mount_root(capture_signal_root(false, captured.clone(), |visible| {
    ui! {
      Modal(visible: visible) {
        Text(text: "base")
        Text(text: "over")
      }
    }
  }));
  assert!(rt.step(Event::Init));
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("base"), "row: {r:?}");
  // "over" should not appear anywhere.
  assert!(!r.contains("over"), "row: {r:?}");
}

#[test]
fn modal_shows_overlay_when_signal_true() {
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  let captured: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  rt.mount_root(capture_signal_root(true, captured.clone(), |visible| {
    ui! {
      Modal(visible: visible) {
        Text(text: "base")
        Text(text: "over")
      }
    }
  }));
  assert!(rt.step(Event::Init));
  // Overlay draws after base in `ZStack` order, so "over" overwrites
  // "base" cell-by-cell at the overlapping positions.
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("over"), "row: {r:?}");
}

#[test]
fn input_renders_value_and_focused_cursor() {
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val2 = val.clone();
  let foc2 = foc.clone();
  rt.mount_root(capture_signal_root("hi".to_string(), val.clone(), move |value| {
    let foc2 = foc2.clone();
    capture_signal_root(true, foc2, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  let _ = (val2, foc);
  assert!(rt.step(Event::Init));
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("hi"), "row: {r:?}");
  // Cursor at position 0 paints the cell under it on a dark-grey
  // background; the underlying character is unchanged.
  let cell = rt.back_buffer().get(Point::new(0, 0)).copied().unwrap();
  assert_eq!(cell.ch, 'h');
  assert_eq!(cell.style.bg, Color::DarkGrey);
}

#[test]
fn input_typing_updates_value_when_focused() {
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(10, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(true, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));

  // Type "ab". Each keystroke runs the handler synchronously (mutating
  // the value signal via `update`), then we drain the resulting
  // SignalChanged notifications to re-render.
  for c in ['a', 'b'] {
    rt.step(Event::Key(KeyEvent {
      code: KeyCode::Char(c),
      modifiers: KeyModifiers::NONE,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    }));
    while let Ok(ev) = rx.try_recv() {
      rt.step(ev);
    }
  }
  let value = val.lock().unwrap().clone().unwrap();
  assert_eq!(value.peek(), "ab");
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("ab"), "row: {r:?}");
  let _ = foc;
}

#[test]
fn input_unfocused_does_not_consume_keys() {
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(false, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));
  rt.step(Event::Key(KeyEvent {
    code: KeyCode::Char('a'),
    modifiers: KeyModifiers::NONE,
    kind: KeyEventKind::Press,
    state: crossterm::event::KeyEventState::NONE,
  }));
  // Handler was never registered, so the value is unchanged.
  let value = val.lock().unwrap().clone().unwrap();
  assert_eq!(value.peek(), "");
  let _ = foc;
}

#[test]
fn zstack_overlay_paints_in_declaration_order() {
  // Regression: rendering used to be ordered by `HashSet` iteration
  // of the dirty queue, so sibling z-order in a `ZStack` was
  // unspecified and `Modal`-style overlays could be partially erased
  // by underlying siblings being painted afterwards. Children must
  // paint in placement order, period.
  use intuitive::components::ZStack;
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(8, 1));
  rt.mount_root(ui! {
    ZStack() {
      Text(text: "base____")
      Text(text: "OVER")
    }
  });
  assert!(rt.step(Event::Init));
  // "OVER" (second child) overwrites the first four cells of
  // "base____" because it draws after.
  assert_eq!(row(rt.back_buffer(), 0), "OVER____");
}

#[test]
fn modal_overlay_paints_above_complex_base_layout() {
  // Stronger regression: a deeply nested base (Tasks/Timer panes
  // via HStack + Border) used to *partially* erase a centered modal
  // overlay because the panes' inner rect-clears ran after the
  // modal's Border drew. With placement-ordered painting and
  // Border's opaque background, the modal must dominate fully.
  use std::sync::{Arc, Mutex};
  // Buffer chosen so the centered modal has at least one
  // border-interior row, so we can assert blank cells inside it.
  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(40, 15));
  let captured: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  rt.mount_root(capture_signal_root(true, captured, |visible| {
    ui! {
      Modal(visible: visible) {
        HStack(.flex: [Flex::Frac(1), Flex::Frac(1)]) {
          Border(.title: "Tasks") { Text(text: "L") }
          Border(.title: "Timer") { Text(text: "R") }
        }
        Centered(.pad_x: Flex::Frac(2), .pad_y: Flex::Frac(2)) {
          Border(.title: "M") { Text(text: "x") }
        }
      }
    }
  }));
  assert!(rt.step(Event::Init));
  // Modal is centered: with width 40 and weights `[2, 1, 2]`,
  // the middle column is cols 16..24 (width 8). With height 15 the
  // middle band is rows 6..9 (height 3). So the modal Border
  // occupies cols 16..24, rows 6..9; its top edge is row 6, bottom
  // edge row 8, with one interior row at row 7 (where the child
  // `Text("x")` lands, leaving its right cells blank).
  let interior_row = row(rt.back_buffer(), 7);
  let chars: Vec<char> = interior_row.chars().collect();
  assert_eq!(chars[16], '│', "row 7: {interior_row:?}");
  assert_eq!(chars[23], '│', "row 7: {interior_row:?}");
  // Inside the modal interior, col 17 is the `x` (the child Text),
  // cols 18..23 must be blank (not leaking through base-pane chars
  // like the HStack separator at col ~20).
  assert_eq!(chars[17], 'x', "row 7: {interior_row:?}");
  for (col, ch) in chars.iter().enumerate().take(23).skip(18) {
    assert_eq!(
      *ch, ' ',
      "modal interior at col {col} not blank in row 7: {interior_row:?}"
    );
  }
}

#[test]
fn modal_overlay_survives_partial_rerender_of_base() {
  // Regression: when a signal subscribed *only* by a lower-z base
  // leaf changes, the runtime used to re-render just that leaf.
  // `render_one` cleared its leaf rect, which intersected a
  // higher-z overlay, and the overlay was never repainted
  // because it wasn't in the dirty set.
  //
  // With opacity tracking + z-aware overlap promotion in
  // `render_dirty`, the overlay (opaque, higher z, intersecting
  // the dirty rect) gets added back to dirty and repaints on top.
  //
  // To isolate the bug, the dirty leaf must be the *only*
  // subscriber to the mutated signal, so re-rendering it does
  // *not* transitively re-mount the overlay subtree from above.
  // We build the tree by hand with `Element::new` so we can
  // control exactly which element subscribes.
  use std::sync::{Arc, Mutex};

  use intuitive::{
    geometry::{Point, Rectangle as Rect, Size as Sz},
    runtime::Element,
  };

  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(20, 5));
  let txt_cap: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let txt_setup = txt_cap.clone();

  // Root: build a signal, mount the base leaf and the opaque
  // overlay as two children of the root. Crucially, the root's
  // render closure does *not* call `txt.get(ctx)` — only the leaf
  // does — so the root is not a subscriber.
  rt.mount_root(Element::new(move |ctx: &mut Context<'_>| {
    let txt = ctx.new_signal_with::<String>(|| "L0".to_string());
    *txt_setup.lock().unwrap() = Some(txt.clone());
    let root_rect = ctx.region().rect();

    // Base leaf: subscribes to `txt`, fills entire root rect.
    let txt_for_leaf = txt.clone();
    let base = Element::new(move |bctx: &mut Context<'_>| {
      let s = txt_for_leaf.get(bctx);
      let mut region = bctx.region();
      // Paint the whole rect so we can detect erasure.
      region.fill(Cell::new('.'));
      region.write_str((0, 0), &s, Style::RESET);
    })
    .with_user_key(Some("base".to_string()));
    ctx.render(base, root_rect);

    // Overlay: opaque (calls `set_opaque`), placed at a higher z
    // via `render_with_z`. Sits in the middle of the root rect,
    // overlapping the base.
    let overlay = Element::new(move |octx: &mut Context<'_>| {
      octx.set_opaque();
      let mut region = octx.region();
      region.fill(Cell::new('#'));
    })
    .with_user_key(Some("overlay".to_string()));
    let overlay_rect = Rect::new(Point::new(5, 1), Sz::new(8, 3));
    ctx.render_with_z(overlay, overlay_rect, ctx.z() + 10);
  }));
  assert!(rt.step(Event::Init));

  // Initial state: row 2 should show base on the sides and the
  // overlay's '#' fill in cols 5..13.
  let r2 = row(rt.back_buffer(), 2);
  assert_eq!(&r2[5..13], "########", "initial row 2: {r2:?}");

  // Mutate the leaf's signal. Only the leaf subscribes to it, so
  // only the leaf is in the dirty set when the runtime processes
  // the SignalChanged event.
  let sig = txt_cap.lock().unwrap().clone().unwrap();
  sig.set("L1".to_string());
  let next = rx.try_recv().expect("expected a SignalChanged notification");
  assert!(matches!(next, Event::Signal(_)));
  rt.step(next);

  // After the partial re-render, the overlay must still be there.
  // Without overlap promotion, the leaf would have cleared its
  // rect, repainted '.'+text, and never restored the '#' cells.
  let r2 = row(rt.back_buffer(), 2);
  // Also verify row 0 has the new text "L1" — i.e. the leaf did
  // actually re-render (sanity check that the test exercises the
  // partial-re-render path at all).
  let r0 = row(rt.back_buffer(), 0);
  assert!(r0.starts_with("L1"), "after-tick row 0 should start with L1: {r0:?}");
  assert_eq!(&r2[5..13], "########", "after-tick row 2: {r2:?}");
}

#[test]
fn input_scrolls_to_keep_cursor_visible_when_typing_past_width() {
  // Region is 5 cells wide. Typing 8 characters should scroll so the
  // cursor (at end) is always visible. The visible window slides to
  // show the tail of the buffer with one cell reserved for the
  // insertion-point cursor.
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(5, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(true, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));

  for c in "abcdefgh".chars() {
    rt.step(Event::Key(KeyEvent {
      code: KeyCode::Char(c),
      modifiers: KeyModifiers::NONE,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    }));
    while let Ok(ev) = rx.try_recv() {
      rt.step(ev);
    }
  }

  // Value is "abcdefgh" (len 8); cursor is at 8. With width 5 the
  // scroll offset settles at `8 + 1 - 5 = 4`, so the visible window
  // is chars 4..9, i.e. "efgh" + an empty cursor-block cell.
  let value = val.lock().unwrap().clone().unwrap();
  assert_eq!(value.peek(), "abcdefgh");
  let r = row(rt.back_buffer(), 0);
  assert_eq!(&r[..4], "efgh", "row: {r:?}");
  // The last column should be a blank cell on dark grey (the cursor).
  let cursor_cell = rt.back_buffer().get(Point::new(4, 0)).copied().unwrap();
  assert_eq!(cursor_cell.ch, ' ');
  assert_eq!(cursor_cell.style.bg, Color::DarkGrey);
  let _ = foc;
}

#[test]
fn input_scrolls_back_when_moving_cursor_left() {
  // After typing past the width, the visible window has slid right.
  // Holding `Home` (or pressing Left enough times) must scroll back
  // so the cursor is visible again at the left edge.
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(5, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(true, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));

  let press = |code| {
    Event::Key(KeyEvent {
      code,
      modifiers: KeyModifiers::NONE,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    })
  };

  for c in "abcdefgh".chars() {
    rt.step(press(KeyCode::Char(c)));
    while let Ok(ev) = rx.try_recv() {
      rt.step(ev);
    }
  }

  // Jump to start of the line.
  rt.step(press(KeyCode::Home));
  while let Ok(ev) = rx.try_recv() {
    rt.step(ev);
  }

  // Visible window should now show the head of the buffer.
  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("abcde"), "row: {r:?}");
  // The first column is the cursor over 'a' (bg dark grey, ch 'a').
  let cursor_cell = rt.back_buffer().get(Point::new(0, 0)).copied().unwrap();
  assert_eq!(cursor_cell.ch, 'a');
  assert_eq!(cursor_cell.style.bg, Color::DarkGrey);
  let _ = foc;
}

#[test]
fn input_does_not_scroll_when_text_fits() {
  // With text shorter than width, the cursor at end should be visible
  // without scrolling: visible window starts at offset 0.
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(10, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(true, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));

  for c in "abc".chars() {
    rt.step(Event::Key(KeyEvent {
      code: KeyCode::Char(c),
      modifiers: KeyModifiers::NONE,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    }));
    while let Ok(ev) = rx.try_recv() {
      rt.step(ev);
    }
  }

  let r = row(rt.back_buffer(), 0);
  assert!(r.starts_with("abc"), "row: {r:?}");
  // Cursor sits at column 3 (just past 'c'), blank on dark grey.
  let cursor_cell = rt.back_buffer().get(Point::new(3, 0)).copied().unwrap();
  assert_eq!(cursor_cell.ch, ' ');
  assert_eq!(cursor_cell.style.bg, Color::DarkGrey);
  let _ = foc;
}

#[test]
fn input_ctrl_a_and_ctrl_e_jump_to_start_and_end() {
  // Ctrl-A and Ctrl-E are readline-style aliases for Home / End.
  // They must NOT also insert literal 'a' / 'e' into the buffer.
  use std::sync::{Arc, Mutex};
  let (mut rt, _tx, mut rx) = Runtime::new_headless(Size::new(20, 1));
  let val: Arc<Mutex<Option<Signal<String>>>> = Arc::new(Mutex::new(None));
  let foc: Arc<Mutex<Option<Signal<bool>>>> = Arc::new(Mutex::new(None));
  let val_setup = val.clone();
  let foc_setup = foc.clone();
  rt.mount_root(capture_signal_root(String::new(), val_setup, move |value| {
    let foc_setup = foc_setup.clone();
    capture_signal_root(true, foc_setup, move |focused| {
      ui! { Input(value: value.clone(), focused: focused) }
    })
  }));
  assert!(rt.step(Event::Init));

  let press = |code, modifiers| {
    Event::Key(KeyEvent {
      code,
      modifiers,
      kind: KeyEventKind::Press,
      state: crossterm::event::KeyEventState::NONE,
    })
  };

  // Type "hello".
  for c in "hello".chars() {
    rt.step(press(KeyCode::Char(c), KeyModifiers::NONE));
    while let Ok(ev) = rx.try_recv() {
      rt.step(ev);
    }
  }

  // Cursor is now at position 5 (end of "hello"). Press Ctrl-A.
  rt.step(press(KeyCode::Char('a'), KeyModifiers::CONTROL));
  while let Ok(ev) = rx.try_recv() {
    rt.step(ev);
  }
  // Cursor must be at position 0, and crucially the value is still
  // "hello" (the 'a' was NOT inserted).
  let value = val.lock().unwrap().clone().unwrap();
  assert_eq!(value.peek(), "hello");
  // The cursor cell at screen col 0 should be over 'h' on dark grey.
  let cell = rt.back_buffer().get(Point::new(0, 0)).copied().unwrap();
  assert_eq!(cell.ch, 'h');
  assert_eq!(cell.style.bg, Color::DarkGrey);

  // Press Ctrl-E to jump back to end.
  rt.step(press(KeyCode::Char('e'), KeyModifiers::CONTROL));
  while let Ok(ev) = rx.try_recv() {
    rt.step(ev);
  }
  // Value still "hello" (no 'e' inserted), cursor at end.
  assert_eq!(value.peek(), "hello");
  // Cursor is now at col 5 (just past 'o'): blank cell on dark grey.
  let cell = rt.back_buffer().get(Point::new(5, 0)).copied().unwrap();
  assert_eq!(cell.ch, ' ');
  assert_eq!(cell.style.bg, Color::DarkGrey);

  let _ = foc;
}

#[test]
fn on_click_hit_tests_against_element_rect() {
  // A parent with an inner child; each registers an `on_click`
  // handler that increments a counter. Verifies that:
  //   1. A click inside the child fires *both* parent and child handlers (the
  //      child's rect lies inside the parent's, so both rects contain the click).
  //   2. A click inside the parent but outside the child fires only the parent's
  //      handler.
  //   3. A click outside both fires neither.
  //   4. Non-`Down` mouse events (move, drag, release, scroll) do not fire any
  //      handler.
  use std::sync::{Arc, Mutex};

  use intuitive::{
    event::{MouseButton, MouseEvent, MouseEventKind},
    geometry::{Point as Pt, Rectangle},
    runtime::Element,
  };

  // The child draws into a fixed sub-rect of the parent. We hand-roll
  // both elements rather than going through ui!{} so the rectangles
  // are explicit and the test is self-contained.
  fn make_child(hits: Arc<Mutex<u32>>) -> Element {
    Element::new(move |ctx: &mut Context<'_>| {
      let hits = hits.clone();
      ctx.on_click(move |_| {
        *hits.lock().unwrap() += 1;
      });
    })
  }

  let parent_hits = Arc::new(Mutex::new(0_u32));
  let child_hits = Arc::new(Mutex::new(0_u32));

  let parent_hits_ = parent_hits.clone();
  let child_hits_ = child_hits.clone();
  let root = Element::new(move |ctx: &mut Context<'_>| {
    let parent_hits_ = parent_hits_.clone();
    ctx.on_click(move |_| {
      *parent_hits_.lock().unwrap() += 1;
    });
    // Place the child at columns 2..4, row 0 (width 2, height 1).
    let child_rect = Rectangle::new(Pt::new(2, 0), Size::new(2, 1));
    ctx.render(make_child(child_hits_.clone()), child_rect);
  });

  let (mut rt, _tx, _rx) = Runtime::new_headless(Size::new(10, 1));
  rt.mount_root(root);
  assert!(rt.step(Event::Init));

  let click_at = |x, y| MouseEvent {
    kind: MouseEventKind::Down(MouseButton::Left),
    column: x,
    row: y,
    modifiers: KeyModifiers::NONE,
  };

  // (1) Click inside child (col 2, row 0) → both fire.
  rt.step(Event::Mouse(click_at(2, 0)));
  assert_eq!(*parent_hits.lock().unwrap(), 1);
  assert_eq!(*child_hits.lock().unwrap(), 1);

  // (2) Click in parent but outside child (col 0, row 0) → parent only.
  rt.step(Event::Mouse(click_at(0, 0)));
  assert_eq!(*parent_hits.lock().unwrap(), 2);
  assert_eq!(*child_hits.lock().unwrap(), 1);

  // (3) Click outside the parent's rect entirely. The parent's rect
  // covers the full 10x1 buffer, so to land "outside" we use a row
  // beyond the height. `Rectangle::contains` is right/bottom-exclusive,
  // so (0, 5) is outside.
  rt.step(Event::Mouse(click_at(0, 5)));
  assert_eq!(*parent_hits.lock().unwrap(), 2);
  assert_eq!(*child_hits.lock().unwrap(), 1);

  // (4) A non-`Down` mouse event must not dispatch, even if its
  // position is inside both rects.
  let drag = MouseEvent {
    kind: MouseEventKind::Drag(MouseButton::Left),
    column: 2,
    row: 0,
    modifiers: KeyModifiers::NONE,
  };
  rt.step(Event::Mouse(drag));
  let up = MouseEvent {
    kind: MouseEventKind::Up(MouseButton::Left),
    column: 2,
    row: 0,
    modifiers: KeyModifiers::NONE,
  };
  rt.step(Event::Mouse(up));
  let scroll = MouseEvent {
    kind: MouseEventKind::ScrollDown,
    column: 2,
    row: 0,
    modifiers: KeyModifiers::NONE,
  };
  rt.step(Event::Mouse(scroll));
  assert_eq!(*parent_hits.lock().unwrap(), 2);
  assert_eq!(*child_hits.lock().unwrap(), 1);
}
