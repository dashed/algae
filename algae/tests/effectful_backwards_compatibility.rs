#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: `#[effectful]` without a `root` argument.
//!
//! Adding the `root = ...` argument must not change how the bare attribute
//! behaves. Code written before the argument existed keeps compiling, keeps
//! taking its own parameters, and still produces `Effectful<T, Op>`.

use algae::prelude::*;
use std::sync::{Arc, Mutex};

effect! {
    Console::Print (String) -> ();
    Math::Add ((i32, i32)) -> i32;
}

#[effectful]
fn traditional_function(x: i32, y: i32) -> String {
    let _: () = perform!(Console::Print("Computing...".to_string()));
    let result: i32 = perform!(Math::Add((x, y)));
    let _: () = perform!(Console::Print(format!("Result: {result}")));
    format!("Answer: {result}")
}

struct TraditionalHandler {
    printed: Arc<Mutex<Vec<String>>>,
}

impl Handler<Op> for TraditionalHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                self.printed.lock().unwrap().push(msg.clone());
                Box::new(())
            }
            Op::Math(Math::Add((a, b))) => Box::new(a + b),
        }
    }
}

#[test]
fn bare_effectful_still_takes_arguments_and_performs_in_order() {
    let printed = Arc::new(Mutex::new(Vec::new()));

    let result = traditional_function(15, 25)
        .handle(TraditionalHandler {
            printed: Arc::clone(&printed),
        })
        .run();

    assert_eq!(result, "Answer: 40");
    assert_eq!(
        printed.lock().unwrap().as_slice(),
        ["Computing...", "Result: 40"]
    );
}

/// The return type stays `Effectful<String, Op>`; coercing to an explicit `fn`
/// pointer fails to compile if it drifts.
#[test]
fn bare_effectful_still_returns_the_default_root() {
    let _f: fn(i32, i32) -> Effectful<String, Op> = traditional_function;
}
