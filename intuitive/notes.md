# invariants
- `State<T>` cannot be a parameter to a struct implementing `Component`
  - this might only be true of the root component
- `use_state` (and other hooks) can only be called in `render`
- if you take children in your custom component you must call `render` on them when being rendered
- frozen / rendred structs should only take in `AnyElement` not `AnyComponent`
- all components should take in a `on_key` parameter
- custom components must implement `Default`

# todo
- add a conditional component, or `if` syntax to `render!`
- consider giving `KeyHandler` an additional parameter, specifically a struct that
  holds state that the `KeyHandler` can inspect. this could handle `onEnter` use
  case for input text boxes.
- create a `use_memo` hook, as it
  1. is probably useful
  2. could serve as the primitive the `use_state`
