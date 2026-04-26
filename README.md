# Intuitive

Intuitive is a component-based library for building text-based user
interfaces (TUIs) in Rust. It is heavily inspired by [React], [SwiftUI], and
[SolidJS] ideas like functional components, hooks, a (mostly) declarative
DSL for describing UI, and signals.

## Design

The main focus of Intuitive is to simplify the implementation of
layout-oriented TUIs (like [lazygit]), even at the slight expense of
performance. It aims to make it easy to write reusable TUI components that:

- encapsulate logic around handling state and key events,
- have non-trivial tiling layouts, and
- are easy to read.

## Tests

Run `nix flake check` to run tests and `nix run .#fixtures` to regenerate
macro fixtures.

[React]: https://reactjs.org/
[SwiftUI]: https://developer.apple.com/xcode/swiftui/
[SolidJS]: https://www.solidjs.com/
[lazygit]: https://github.com/jesseduffield/lazygit
