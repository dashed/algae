#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: `#[effectful]` against a custom root type.
//!
//! `#[effectful]` used to hardcode `Op` as the root, so effects declared with
//! `effect! { root CustomOp; ... }` could not be driven by it. The attribute now
//! accepts `#[effectful(root = CustomOp)]`, and a second effect family in the
//! same file confirms the bare attribute still defaults to `Op`.

use algae::prelude::*;
use std::sync::{Arc, Mutex};

effect! {
    root CustomOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
    Math::Add ((i32, i32)) -> i32;
}

#[effectful(root = CustomOp)]
fn greet_and_calculate(x: i32, y: i32) -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);

    let _: () = perform!(Console::Print(format!(
        "Hello, {name}! Let me calculate {x} + {y}"
    )));
    let result: i32 = perform!(Math::Add((x, y)));

    let _: () = perform!(Console::Print(format!("The result is: {result}")));

    format!("{name}: {x} + {y} = {result}")
}

effect! {
    SimpleOps::GetValue -> i32;
}

#[effectful] // Should default to Op
fn simple_function() -> i32 {
    perform!(SimpleOps::GetValue)
}

struct CustomHandler {
    input: String,
    printed: Arc<Mutex<Vec<String>>>,
}

impl Handler<CustomOp> for CustomHandler {
    fn handle(&mut self, op: &CustomOp) -> Box<dyn std::any::Any + Send> {
        match op {
            CustomOp::Console(Console::Print(msg)) => {
                self.printed.lock().unwrap().push(msg.clone());
                Box::new(())
            }
            CustomOp::Console(Console::ReadLine) => Box::new(self.input.clone()),
            CustomOp::Math(Math::Add((a, b))) => Box::new(a + b),
        }
    }
}

struct SimpleHandler;

impl Handler<Op> for SimpleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::SimpleOps(SimpleOps::GetValue) => Box::new(42),
        }
    }
}

#[test]
fn effectful_drives_a_custom_root() {
    let printed = Arc::new(Mutex::new(Vec::new()));

    let result = greet_and_calculate(10, 20)
        .handle(CustomHandler {
            input: "Alice".to_string(),
            printed: Arc::clone(&printed),
        })
        .run();

    assert_eq!(result, "Alice: 10 + 20 = 30");
    assert_eq!(
        printed.lock().unwrap().as_slice(),
        [
            "What's your name?",
            "Hello, Alice! Let me calculate 10 + 20",
            "The result is: 30"
        ]
    );
}

#[test]
fn bare_effectful_alongside_a_custom_root_still_defaults_to_op() {
    let result = simple_function().handle(SimpleHandler).run();

    assert_eq!(result, 42);
}

/// Each function must be generated against the root it asked for; coercing to
/// an explicit `fn` pointer fails to compile if either signature drifts.
#[test]
fn generated_signatures_carry_the_expected_root_type() {
    let _custom: fn(i32, i32) -> Effectful<String, CustomOp> = greet_and_calculate;
    let _simple: fn() -> Effectful<i32, Op> = simple_function;
}
