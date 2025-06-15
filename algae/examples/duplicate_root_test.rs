//! Test that demonstrates the sentry mechanism for detecting duplicate root names.
//! 
//! This file intentionally contains conflicting effect! declarations to show
//! how the sentry enum mechanism provides clear error messages.
//! 
//! IMPORTANT: This file will NOT compile! It's designed to show the error.
//! To see the error, run: cargo check --example duplicate_root_test

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// ============================================================================
// ❌ INTENTIONAL ERROR: Duplicate root names
// ============================================================================

// First effect with default root name (Op)
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// Second effect with same default root name (Op) - this will cause an error!
// The sentry enum mechanism will trigger: "duplicate definition of `__ALGAE_EFFECT_SENTRY_FOR_Op`"
effect! {
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
}

// The above code will produce a compile error like:
// error[E0428]: the name `__ALGAE_EFFECT_SENTRY_FOR_Op` is defined multiple times
//   --> examples/duplicate_root_test.rs:XX:YY
//    |
// XX |   effect! {
//    |   ^^^^^^^^ `__ALGAE_EFFECT_SENTRY_FOR_Op` redefined here
//    |
// XX |   effect! {
//    |   -------- previous definition of the type `__ALGAE_EFFECT_SENTRY_FOR_Op` here
//    |
//    = note: `__ALGAE_EFFECT_SENTRY_FOR_Op` must be defined only once in the type namespace of this module

// ============================================================================
// ✅ SOLUTION: Use different root names
// ============================================================================

// Uncomment these lines instead to see the correct approach:
/*
effect! {
    root ConsoleOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

effect! {
    root MathOp;
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
}
*/

fn main() {
    println!("This example demonstrates the error message for duplicate root names.");
    println!("The compile error provides a clear indication of the problem.");
}