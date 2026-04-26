//! Snapshot tests of macro expansion output.
//!
//! Fixtures in `tests/expand/` are expanded with `cargo-expand` and compared
//! against the corresponding `.expanded.rs` file.

#[test]
fn expand() {
  macrotest::expand("tests/expand/*.rs");
}
