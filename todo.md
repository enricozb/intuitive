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
    - **this is a different notion of parents than when unmounting**
      if we have a root component that renders `Section() { Text() }`, the root element's immediate
      children (in the unmounting sense) are `Section` and `Text`. However, in the sense of event
      propagation, `Text` is a child of `Section`, and `Section` a child of the root element.
      - perhaps the `render` function shoudl take care of this? Maybe we have to manaully pass in a components parent id?
        - nah this won't work, because a component can be rendered not immediately in the context of its parent, and then
          passed in later:
          ```
          let el = render! { SomeComponent() };
          render! { Section() { Embed(el) } }
          ```
        - nah this would work, b/c the second `render!` call could call `render()` on `Section`, and apply this `set_parent`
          to the provided child, `Embed`.
        - but it's a little hard to introspect an element's children within `render()`, and might be easier within `render!`
      - perhaps the `render!` macro can take this. Any time we pass in children, we call a function on `Children<N>`, like
        `Children::register_parent` or something.
        - or we read the IDs from the children manually and register them then
          - i _think_ this could work, because children being prop drilled will have this called multiple times on them,
            the last one being on the parent they ended up with

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
  - signals: finer-grained re-rendering) on `State` changes

- rendering
  - unmount
    - elements should be "unmounted": when a re-render occurs for a parent of an element
      and this element _isn't_ re-rendered, it should be unmounted
    - unmounting an element could:
      - call an `on_umount` method on the element
      - have `Initializer` return something that implements `Unmountable`, so we can call `unmount` on
        `State`, and other structs returned by `use_hook`.
        - `Memos` would have to be changed to store `dyn Unmountable` (better named `Hook`)
      - delete the hooks from the global `COMPONENTS` and `ELEMENTS`

- misc
  - remove mutexes, arcs, `Sync` and `Send` requirements by moving away from `lazy_static` and `static ref`, and instead
    making the terminal own an instance of a render manager.
    - this might be harder than i originally thought. Mostly because `render!` calls `render::render`, which is a top-level function.
      I'm not sure how to make `render!` generate code that refers to the current terminal.
    - Maybe `Component::render` can take in a `Terminal` reference.
  - implement proper tracing
  - `Terminal::render` should ensure that cleanup is called before it leaves that function, or else error isn't printed
  - cursor showing/hiding ability (or leave this to crossterm?)

- layout
  - when laying things out in `Stack`, we need to compute the remaiing space across `Grow` cases, and spread out
    the remainder evenly among the first `k` `Grow` children.

- tests
  - `use_state`
  - `use_effect`

- docs
  - hooks
    - writing custom hooks
