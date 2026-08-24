//! `call!` outside an `#[effectful]` function must produce one clear
//! message, not a wall of coroutine trait errors.
#![feature(coroutines, yield_expr)]
use algae::prelude::*;

effect! {
    Test::GetValue -> i32;
}

#[effectful]
fn subtask() -> i32 {
    perform!(Test::GetValue)
}

fn not_effectful() -> i32 {
    call!(subtask())
}

fn main() {}
