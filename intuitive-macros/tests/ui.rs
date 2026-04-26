//! End-to-end compilation tests for the `intuitive-macros` crate.
//!
//! Fixtures in `tests/pass/` must compile successfully. Fixtures in
//! `tests/fail/` must fail to compile, and their emitted diagnostics are
//! compared against the corresponding `.stderr` file.

#[test]
fn ui() {
  let t = trybuild::TestCases::new();
  t.pass("tests/pass/*.rs");
  t.compile_fail("tests/fail/*.rs");
}
