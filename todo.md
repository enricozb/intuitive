- styles

- events
  - focusing system

- macros
  - `render!`
    - add the ability to specify generic parameters when using components, using turbofish:
      ```rust
      render! {
        Table::<3>()
      }
      ```
  - `on_key!`
  - `on_mouse!`

- components
  - `Text` multi-line string
  - input

- hooks
  - `use_memo`
  - `use_effect`
    - having a `deps: D` arg is a little difficult:
      - if the depenedency is a `State`, then just saving a clone of it wouldn't let us compare it to future
        versions of the state because the clone and the "future" version would hold `Arc`s to the same point
        in memory, so they would always be equal.
  - signals: finer-grained re-rendering) on `State` changes

- rendering
  - unmount
    - elements should be "unmounted": when a re-render occurs for a parent of an element
      and this element _isn't_ re-rendered, it should be unmounted
    - unmounting an element could:
      - call an `on_umount` method on the element
      - have `Initializer` return something that implements `Unmountable`, so we can call `unmount` on
        `State`, and other structs returned by `use_hook`.
      - delete the hooks from the global `COMPONENTS` and `ELEMENTS`

- misc
  - `Terminal::render` should ensure that cleanup is called before it leaves that function, or else error isn't printed
  - cursor showing/hiding ability (or leave this to crossterm?)

- layout
  - add `FlexArray<N>` and `Children<N>` so we can derive `Default` for these.
  - when laying things out in `Stack`, we need to compute the remaiing space across `Grow` cases, and spread out
    the remainder evenly among the first `k` children.

- tests
  - `use_state`
  - `use_effect`

- docs
  - macros
