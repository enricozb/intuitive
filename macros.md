# Intuitive — Macros

This document specifies the two proc macros that make up Intuitive's
declarative front-end:

- `#[component]` turns a function definition into a component
  **type** — a plain struct holding the required arguments
  (“props”), plus a sibling builder struct with the optional-arg
  setters and a final `build()`.
- `ui!{ ... }` is the SwiftUI-style call-site syntax that lowers
  into `Name { .. }.new().setter(..)..build()` chains.

The two macros are designed together: `ui!{}` knows nothing about
component signatures, but it emits a lowering that fits the shape
`#[component]` always produces. As long as both sides agree on that
shape, the macros are independent and either one can in principle be
swapped out for hand-written code.

See `design.md` for the underlying runtime that the produced
`Element` values are fed into.

---

## The component contract

Every component the macros produce exposes two types and two impl
blocks:

- A **props struct** `Name` whose `pub` fields are exactly the
  required arguments. Constructing a value of this struct is what
  users write at the call site (through `ui!{}`) to supply required
  arguments: `Name { req: e, .. }`. Missing required arguments are
  a compile error of the form "missing field `text`" — the standard
  Rust struct-literal diagnostic. The struct is always emitted in
  braced form (`pub struct Name { .. }`, never a unit struct) so
  that `Name { }` is a valid literal even when the component takes
  no required arguments.
- A **builder struct** `NameBuilder` carrying *every* parameter
  (required + optional). This is the type that holds setter state
  and produces the final `Element`.
- `impl Name`: a single inherent method `fn new(self) -> NameBuilder`,
  which destructures the props struct, fills in defaults for every
  optional parameter, and returns a builder.
- `impl NameBuilder`: one inherent setter per optional argument
  (each taking `self` by value and returning `Self`), plus
  `fn build(self) -> Element` which runs the original function body.

There is **no trait**. `new`, the setters, and `build` are all
inherent methods, and `ui!{}` calls them by plain name. The shape
was chosen so that the user only needs `Name` in scope: writing
`Name { .. }` constructs the props value without a separate
`NameProps` identifier ever appearing in user code. The configured
component (with optional-arg setters applied) lives in the sibling
`NameBuilder` type, which user code never names directly — it only
appears as an intermediate type in the `ui!{}` lowering.

Different optional setters have different signatures (the type of
the optional argument), so they could not share a trait method
anyway — they always have to live in the inherent-method namespace.

> Historical note. `Name` used to be emitted as a single-variant
> `enum Name { Props { .. } }`, written specifically so the call
> site could use the path-qualified literal `Name::Props { .. }`
> as a stable stand-in for an inherent associated type. Since the
> chosen call-site surface is `Name { .. }` directly, the enum
> workaround is no longer needed and `Name` is now a plain struct.
> The rustdoc output is correspondingly more honest: `Name` shows
> up as exactly what it is, a bag of required props.

---

## `#[component]` attribute macro

### Surface

A `#[component]`-annotated function declares a component. Three
shapes are accepted, distinguished by whether the function takes a
parameter named `ctx` and what return type (if any) it declares:

```rust
// Pure composition. Body produces an Element value.
#[intuitive::component]
pub fn TasksPane(focused: bool, /* ... */) -> Element {
  ui! { Border(.focused: focused) { /* ... */ } }
}

// Composition with effects. Body runs ctx-effects (signal reads,
// `on_key`, `spawn`, ...) and evaluates to an `Element`; the macro
// auto-renders it into the component's own region.
#[intuitive::component]
pub fn TasksBody(ctx: &mut Context<'_>, /* ... */) -> Element {
  let sel = selected.get(ctx);
  ctx.on_key(/* ... */);
  ui! { List(items: /* ... */, .selected: sel) }
}

// Raw renderer. Body IS the render closure: it draws into
// `ctx.region()` and/or calls `ctx.render(..)` directly.
#[intuitive::component]
pub fn Text(
  ctx: &mut Context<'_>,
  text: String,                          // required
  #[default = "Color::Reset"] fg: Color, // optional, explicit default
  #[default] bold: bool,                 // optional, Default::default()
) {
  let mut region = ctx.region();
  region.fill(Cell::BLANK);
  // ... draw `text` ...
}
```

See "Body modes" below for the precise rules.

Parameter attributes:

- **(none), name != `ctx`** → **required**. The parameter becomes a
  `pub` field of the `Name` props struct, must be supplied at the
  call site, and is forwarded into the builder by `new`.
- **(none), name == `ctx`** → **render-context**. The parameter is
  *not* a prop; instead, the macro wraps the body in a render
  closure that binds `ctx` to `&mut Context<'_>`. See "Body modes".
- `#[default]` → **optional**, initial value is
  `::core::default::Default::default()`. Requires `T: Default`.
- `#[default = "expr"]` → **optional**, initial value is the Rust
  expression parsed from the string literal. The expression is
  spliced into the generated `new` body, so any names it mentions
  must be in scope at the component's definition site.
- `#[children]` → the **children-factory** slot. See "Children
  parameter" below.

### Lowering

For a leaf component (no `#[children]` param) like the `Text` above,
`#[component]` emits roughly:

```rust
pub struct Text {
  pub text: String,             // required-only; `pub` so the
                                // literal is constructible from
                                // outside the defining module.
}

pub struct TextBuilder {
  text: String,                 // builder fields are not `pub` —
  fg: Color,                    // the builder is not user-
  bold: bool,                   // constructible directly.
}

impl Text {
  pub fn new(self) -> TextBuilder {
    let Self { text } = self;
    TextBuilder {
      text,
      fg:   Color::Reset,
      bold: ::core::default::Default::default(),
    }
  }
}

impl TextBuilder {
  pub fn build(self) -> Element {
    let Self { text, fg, bold } = self;
    // original function body verbatim
  }

  #[must_use] pub fn fg(mut self, value: Color) -> Self { self.fg = value; self }
  #[must_use] pub fn bold(mut self, value: bool) -> Self { self.bold = value; self }
}
```

The props struct keeps the component's PascalCase name; the builder
mirrors that name with a `Builder` suffix (`TextBuilder`). Both are
emitted at the visibility of the original `fn` (so `pub fn Text` →
`pub struct Text` + `pub struct TextBuilder`), as are `new`, `build`,
and the optional-arg setters. The required fields of the props
struct additionally carry that same visibility on each field, so
`Name { req: e }` literals work from any module that can name `Name`.

### Children parameter

A parameter annotated `#[children]` is special-cased. The macro
recognizes it and emits a builder method matching its type:

```rust
#[intuitive::component]
pub fn HStack(
  ctx: &mut Context<'_>,
  flex: Vec<Flex>,
  #[children] children: Vec<Element>,
) {
  let kids = children();    // factory call, see below
  // ... layout & ctx.render(..) for each kid ...
}
```

(This uses the `ContextRender` body mode — `ctx + no return type`
— because container components draw via `ctx.render(..)` calls
rather than producing a single `Element` value.)

The user declares the children parameter as the **element type the
body consumes** (`Vec<Element>` for n-ary containers; could be
`Element` in the future for strict single-child containers). The
generated code stores the boxed factory:

```rust
pub struct HStack {
  pub flex: Vec<Flex>,
}

pub struct HStackBuilder {
  flex: Vec<Flex>,
  children: Option<Box<dyn FnMut() -> Vec<Element>>>,
}

impl HStackBuilder {
  // ... per-optional-arg setters live here too ...

  pub fn children<F>(mut self, factory: F) -> Self
  where F: FnMut() -> Vec<Element> + 'static,
  { self.children = Some(Box::new(factory)); self }

  pub fn build(self) -> Element {
    let Self { flex, children: __children_factory } = self;
    let mut children = __children_factory
      .expect("component `HStack` requires `.children(..)` to be set at the call site");
    // user body runs here, with `children` rebound as the
    // unwrapped factory `Box<dyn FnMut() -> Vec<Element>>`.
  }
}
```

So inside the body the user's `children` is a `FnMut() -> T` callable
once per render. The unwrap in `build()` makes `.children(..)`
effectively required at the call site; calling a `#[children]`-having
component without a `{ ... }` block panics at construction time. (A
future refinement could mark `children` as truly optional via
`#[default]` on the same parameter.)

At most one `#[children]` parameter is allowed.

### Body modes

The presence of a parameter named `ctx` and the declared return
type together select one of three body shapes for `build()`:

| `ctx` param | declared return | body mode             | what `build()` emits                                            |
| ----------- | --------------- | --------------------- | --------------------------------------------------------------- |
| no          | `-> Element`    | `ReturnsElement`      | body verbatim (it must evaluate to `Element`)                   |
| no          | omitted         | *error*               | (the macro requires `-> Element` when there's no `ctx`)         |
| yes         | `-> Element`    | `ContextThenElement`  | wraps body in render closure; auto-renders the returned element |
| yes         | omitted         | `ContextRender`       | wraps body in render closure (body is the render code itself)   |

In the two `ctx`-modes the macro emits a `Box::new(move |ctx: <ty>| { .. })`
render closure around the body, so `ctx` is bound inside as the
user declared it. The macro does *not* inspect `ctx`'s declared
type; users are expected to write `&mut Context<'_>`, but any
mismatch surfaces as an ordinary compile error when the closure is
boxed.

`ctx` is excluded from the props struct and the builder. All
other parameters are bound as locals at the top of the body, just
as in the no-`ctx` form.

#### `ContextRender` (raw renderer)

```rust
#[intuitive::component]
pub fn Text(ctx: &mut Context<'_>, text: String) {
  let mut r = ctx.region();
  r.fill(Cell::BLANK);
  r.write_str(0, 0, &text, Color::Reset, false);
}
```

lowers to:

```rust
impl TextBuilder {
  pub fn build(self) -> ::intuitive::context::Element {
    let Self { text } = self;
    Box::new(move |ctx: &mut Context<'_>| {
      let mut r = ctx.region();
      r.fill(Cell::BLANK);
      r.write_str(0, 0, &text, Color::Reset, false);
    })
  }
}
```

#### `ContextThenElement` (composition with effects)

```rust
#[intuitive::component]
pub fn TasksBody(ctx: &mut Context<'_>, focused: bool) -> Element {
  let sel = selected.get(ctx);
  ctx.on_key(/* ... */);
  ui! { List(items: items, .selected: sel) }
}
```

lowers to:

```rust
impl TasksBodyBuilder {
  pub fn build(self) -> ::intuitive::context::Element {
    let Self { focused } = self;
    Box::new(move |ctx: &mut Context<'_>| {
      let __child: Element = {
        let sel = selected.get(ctx);
        ctx.on_key(/* ... */);
        ui! { List(items: items, .selected: sel) }
      };
      let __rect = ctx.region().rect();
      ctx.render(span!(), None, __rect, __child);
    })
  }
}
```

The auto-emitted `ctx.render(..)` draws the returned element into
the component's *own* assigned region (i.e. the rectangle the
parent gave this element). A user who needs to route the element
somewhere else (a sub-rect, a different element key) should fall
back to `ContextRender` and write the `ctx.render` call manually.

#### `ReturnsElement` (pure composition)

```rust
#[intuitive::component]
pub fn TasksPane(focused: bool) -> Element {
  ui! { Border(.focused: focused) { /* ... */ } }
}
```

is emitted unchanged: `build()` returns the user's body verbatim.
No render closure is involved — the produced `Element` will be
run by whichever parent calls `ctx.render(..)` on it.

#### Restrictions on `ctx`

At most one `ctx` parameter is allowed. Combining `ctx` with
`#[default]` or `#[children]` on the same parameter is rejected (a
name-based `ctx` parameter never has classifier attributes).


### Restrictions

`#[component]` does not currently support:

- generic components (`fn Foo<T>(...)`),
- `async` components,
- `self` arguments,
- destructuring patterns in parameter position (only plain
  identifiers).

These are unimplemented rather than fundamentally incompatible.
Return-type rules are described in "Body modes" above: without a
`ctx` parameter, the function must return `Element`; with a `ctx`
parameter, the return type is either `Element`
(`ContextThenElement` mode) or omitted (`ContextRender` mode).

---

## `ui!{}` macro

### Surface grammar

```
ui_input    := child

child       := component | if_child | for_child | match_child

component   := IDENT ('(' arg_list? ')')? children?
arg_list    := arg (',' arg)* ','?
arg         := '.'? IDENT ':' EXPR              // see "expression scope"
children    := '{' child* '}'                    // no separators

if_child    := 'if' EXPR '{' child* '}' else_tail?
else_tail   := 'else' if_child
             | 'else' '{' child* '}'

for_child   := 'for' PAT 'in' EXPR '{' child* '}'
match_child := 'match' EXPR '{' match_arm (',' match_arm)* ','? '}'
match_arm   := PAT ('if' EXPR)? '=>' arm_body
arm_body    := child
             | '{' child* '}'
```

The parenthesized argument list `(...)` is **optional**. When omitted,
the component is treated as having no required and no optional arguments
— equivalent to writing `()`. This is a convenient shorthand for
components whose required props are all absent or defaulted:

```rust
ui! {
  ZStack { A() B() }       // same as ZStack() { A() B() }
  Empty                    // same as Empty()
}
```

The leading `.` on an argument distinguishes **optional** (builder)
arguments from **required** (props-struct) arguments:

```rust
ui! {
  Text(text: "hello", .fg: Color::Red, .bold: true)
}
```

- `text: "hello"` — required arg, lands inside `Text { text: "hello" }`.
- `.fg: Color::Red` — optional arg, lands in `.fg(Color::Red)` on the builder.
- `.bold: true` — same.

The sigil-based design lets `ui!{}` route each arg without knowing
the component's definition. Mismatch (e.g. writing `.text: "hi"`
when `text` is required, or `text: "hi"` when it's optional) becomes
a normal compile error at the generated call: either "no method
named `text` on `TextBuilder`" or "no field `text` on struct `Text`".

### Children are juxtaposed, no separators

Every form in the `child` non-terminal is **self-delimiting** — it
ends at a balanced closing token (`)` or `}`). So the parser can
loop "parse one child, repeat until end of block" without separators
or lookahead.

### Expression scope

Argument values, `if` conditions, `for` iterators, and `match`
scrutinees are parsed with `syn::Expr::parse_without_eager_brace`.
This avoids the standard struct-literal ambiguity: in

```rust
if focus.get(ctx) { Section() }
```

we want `focus.get(ctx)` as the condition and `{ Section() }` as the
then-block. Eager-brace expression parsing would otherwise interpret
the brace as a struct-literal body. Rust itself uses the same trick
in its own grammar for `if`/`while`/`for` conditions.

The same applies to argument expressions: writing
`Panel(.style: Style) { A() }` should not parse `Style { A() }` as a
struct literal. Argument expressions are therefore also
non-eager-brace.

### Lowering

A single component invocation lowers to:

```rust
{
  #[allow(clippy::redundant_field_names)]
  let __element = Name { req1: e1, req2: e2 }
    .new()                                              // -> NameBuilder
    .opt1(v1)
    .opt2(v2)
    .children(move || -> Vec<Element> { /* pushed kids */ })
    .build();
  __element
}
```

A few notes on the shape:

- The call site only needs `Name` in scope. `Name { .. }` is a plain
  struct literal, and every subsequent method is inherent on `Name`
  or `NameBuilder`. User code never has to import or name
  `NameBuilder` explicitly.
- The outer `{ let __element = ... ; __element }` block exists only
  to host a `#[allow(clippy::redundant_field_names)]` annotation: the
  emitted `Name { foo: foo }` pattern is unavoidable for the
  idiom `Foo(foo: foo)`, and clippy's "use shorthand" suggestion is
  not something the user can act on without changing surface syntax.
- `ui!{}` always emits its output inside an expression-statement
  position (the `{ let __element = ...; __element }` block), so the
  bare `Name { .. }` struct literal is never parsed in a non-eager-
  brace context. Users who construct `Name { .. }` *directly* in a
  condition/scrutinee position (e.g. `if Text { text: x }.new()...`)
  would need to parenthesize per ordinary Rust grammar; the `ui!{}`
  surface itself does not expose this corner.

### Control flow inside child lists

`if` / `for` / `match` lower to ordinary Rust control flow inside the
emitted children factory. They are **not** components — they don't
have their own `ElementKey`, and each leaf they ultimately produce
keeps the call-site span it would have had if written directly:

```rust
ui! {
  HStack(flex: vec![Flex::Frac(1), Flex::Frac(1)]) {
    A()
    if some_bool { B() } else { C() }
  }
}
// ↓ (sketch; the `redundant_field_names`-suppression block is omitted)
HStack { flex: vec![Flex::Frac(1), Flex::Frac(1)] }
  .new()
  .children(move || -> Vec<Element> {
    let mut __kids = Vec::new();
    __kids.push(A {}.new().build());
    if some_bool {
      __kids.push(B {}.new().build());
    } else {
      __kids.push(C {}.new().build());
    }
    __kids
  })
  .build();
```

The example yields **two children** for `HStack`, not three: `if` is
a single `child` in the grammar that pushes one element on whichever
branch is taken. The runtime's child-set diff (see `design.md`,
`Runtime::render`) handles identity correctly: when `some_bool`
flips, the previous `B` / `C` (at a distinct call-site span from the
new one) unmounts and the other mounts.

`for` lowers analogously:

```rust
ui! {
  VStack(flex: flex) {
    for task in tasks {
      Row(name: task.name, key: task.id.clone())
    }
  }
}
```

becomes a `for` loop that pushes one element per iteration. Note that
every iteration of a `for` is at the same source span, so the
component must thread a per-iteration user key through to
`ctx.render(..., user, ...)` for distinct `ElementKey`s. `ui!{}` does
nothing magical with a `key`-named arg today; it's just one more
required arg the component author chooses to expose.

### Capturing in factories

The children factory is emitted as `move ||`, so it captures
everything by move. A parent that wants to reuse a binding across
renders should pre-clone it at the `ui!{}` call site:

```rust
ui! {
  HStack(flex: vec![Flex::Frac(1), Flex::Frac(1)]) {
    Tasks(focused: tasks_focused, selected: selected.clone())
    Timer(focused: timer_focused, remaining: remaining.clone())
  }
}
```

The `.clone()`s here clone `Signal<T>` handles (cheap, just `Arc`
bumps), so a fresh handle moves into the factory each render and the
outer closure keeps its own. `ui!{}` does **not** synthesize these
clones — they're explicit and visible.

### Top-level restriction

The top level of `ui!{}` must be a single component invocation.
`if`/`for`/`match` at the top is a compile error: there's no
surrounding `Vec<Element>` to push into, and the macro must produce
a single `Element` value.

---

## What `ui!{}` does **not** do

- It does not introspect component definitions, so it cannot validate
  argument names or types beyond what the emitted code passes to
  rustc. Typos surface as standard "missing field" / "no method"
  errors at the generated struct literal or method chain.
- It does not auto-clone captured signals or props into the factory.
- It does not generate `ctx.render` calls — its output is a value of
  type `Element`, to be passed to `ctx.render` (or to another
  `ui!{}`-using parent) by the surrounding code.
- It does not enforce per-iteration user keys in `for` bodies. The
  runtime will misbehave silently (duplicate `ElementKey`s) if a
  component used inside a `for` does not thread a unique key through
  to `ctx.render`.
- It does not support arbitrary Rust expressions as children. If you
  want to splice a computed `Element` value, you must wrap it in a
  component (typically a thin `#[component]`-annotated wrapper that
  takes the value as a required arg and returns it).

---

## Examples

### Pomodoro root, declaratively

```rust
let stack = ui! {
  HStack(flex: vec![Flex::Frac(1), Flex::Frac(1)]) {
    TasksPane(
      focused: tasks_focused,
      selected: selected.clone(),
      checked: checked.clone(),
    )
    TimerPane(
      focused: timer_focused,
      remaining: remaining.clone(),
      running: running.clone(),
    )
  }
};
```

`HStack(flex: ...)` → required arg. The children block lowers to
`.children(|| vec![...])`, populated by the two pane components.

### A leaf with mixed required and optional args

```rust
ui! {
  Text(text: "hi".into(), .fg: Color::Red, .bold: true)
}
```

Lowers to (roughly) `Text { text: "hi".into() }.new().fg(Color::Red).bold(true).build()`.

### Conditional child

```rust
ui! {
  HStack(flex: vec![Flex::Frac(1), Flex::Frac(1)]) {
    A()
    if some_bool { B() } else { C() }
  }
}
```

Two children for `HStack`. The second one is `B()` or `C()` depending
on `some_bool`; they have distinct spans, so flipping `some_bool`
triggers a clean unmount/mount in the runtime.

### Loop of children

```rust
ui! {
  VStack(flex: flex) {
    for (i, task) in tasks.iter().enumerate() {
      Row(name: task.name.clone(), key: format!("{i}"))
    }
  }
}
```

N children for `VStack`, one per task. `Row` is responsible for
threading its `key` arg through to `ctx.render` for stable element
identity.

---

## Implementation outline

The macros live in `intuitive-macros-core` (a regular library, so
unit-testable) and are re-exported as proc macros from
`intuitive-macros`. The `intuitive` crate re-exports both entry
points (`ui!` and `#[component]`) at the crate root.

`intuitive-macros-core`:

- `expand_ui(input) -> Result<TokenStream>` — parses `ui!{}` syntax
  and emits the chained-call expansion described above.
- `expand_component(_attr, item) -> Result<TokenStream>` — parses a
  `fn` item, classifies each parameter (required / `#[default]` /
  `#[children]`), and emits the props struct, builder struct, and
  the two inherent `impl` blocks (`Name::new` to fill in defaults
  and produce the builder; `NameBuilder::build` plus per-optional-arg
  setters).

`intuitive-macros`:

- `#[proc_macro] fn ui` — shim.
- `#[proc_macro_attribute] fn component` — shim.

Tests:

- Core unit tests (token-stream shape).
- `macrotest` snapshots in `intuitive-macros/tests/expand/`.
- `trybuild` pass/fail cases in `intuitive-macros/tests/pass/` and
  `intuitive-macros/tests/fail/`, compiled against the real
  `intuitive` crate.

### `extern crate self as intuitive`

The macros emit fully-qualified paths like
`::intuitive::context::Element` and
`::intuitive::components::HStack`. So that those paths resolve
both inside the `intuitive` crate itself (where the crate name
`intuitive` would otherwise refer to nothing) and in downstream
crates, `intuitive/src/lib.rs` includes:

```rust
extern crate self as intuitive;
```

This makes the crate's own public name resolve to itself, so the
same emitted paths work in both contexts.
