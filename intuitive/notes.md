# invariants
- `State<T>` cannot be a parameter to a struct implementing `Component`
  - this might only be true of the root component
- `use_state` (and other hooks) can only be called in `render`
- if you take children in your custom component you must call `render` on them when being rendered
- frozen / rendred structs should only take in `AnyElement` not `AnyComponent`
- all components should take in a `on_key` parameter
- custom components must implement `Default`

# todo
- make `Component(prop1: value1, prop2: value2)` syntax where we don't need to specify the prop name if the value name matches,
  like `Component(prop1, prop2)`
