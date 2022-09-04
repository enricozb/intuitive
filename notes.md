# invariants
- `State<T>` cannot be a parameter to a struct implementing `Component`
- `use_state` (and other hooks) can only be called in `render`
- if you take children in your custom component you must call `render` on them when being rendered
