- styles

- events
  - focusing system

- macros
  - `on_key!`
  - `on_mouse!`

- components
  - `Text` multi-line string
  - hstack/vstack
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

- tests
  - `use_state`
  - `use_effect`

- docs
  - macros
