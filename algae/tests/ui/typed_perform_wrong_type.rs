//! With the typed constructors, requesting the wrong reply type is a compile
//! error at the perform site — not a runtime panic in the handler plumbing.
#![feature(coroutines, yield_expr)]
use algae::prelude::*;

effect! {
    Math::Add ((i32, i32)) -> i32;
}

#[effectful]
fn wrong() -> String {
    let s: String = perform!(Math::add(1, 2)); // declared reply type is i32
    s
}

fn main() {}
