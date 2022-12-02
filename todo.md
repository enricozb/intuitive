- styles

- focusing system
  - `element::Any` could have a reference to the `ComponentID` that generated, then assuming that there can
    only be a single element in "focus", and that the `render` module has knowledge of this, we could implement
    `element::Any::is_focused()` and `element::Any::focus()` to query and control focus of elements.

  - question: should `focused` be a property?
      - pros
        - easy for a component/element to check if it is in focus
        - a user could specify which component _should_ be in focus
      - cons
        - users could pass in `focused: true` for all components
        - `focused` should not affect how a component is rendered, only how it is drawn

  - if focused is _not_ a property, how can element know it is focused when drawing?
    - a new param to `draw`
      - `focused: bool`
      - `component_id: ComponentID`
        - then we can check `render::is_focused(component_id)`

  - in order to change focus on click, `render` needs to know which regions map to which elements

  - in order for events to propagate properly, we need a hierarchy of elements
    - `render` needs to know which elements are parents of others

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
  - when laying things out in `Stack`, we need to compute the remaiing space across `Grow` cases, and spread out
    the remainder evenly among the first `k` `Grow` children.

- tests
  - `use_state`
  - `use_effect`

- docs
  - macros
