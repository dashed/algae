#![cfg(feature = "macros")]

//! Compile-fail coverage for the guard rails built into the `effect!` macro.
//!
//! Every file under `tests/ui/` must fail to compile, producing exactly the
//! diagnostics recorded in the sibling `.stderr` snapshot. That snapshot is the
//! real assertion: it is what keeps the error messages users see from silently
//! degrading.
//!
//! Regenerate the snapshots after an intentional change to a diagnostic:
//!
//! ```text
//! TRYBUILD=overwrite cargo test -p algae --test compile_fail
//! ```

#[test]
fn ui() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
