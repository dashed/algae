//! Test that #[effectful] maintains backwards compatibility
//!
//! This example verifies that existing code using #[effectful] without
//! the root argument continues to work exactly as before.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Traditional effect definition (no custom root)
effect! {
    Console::Print (String) -> ();
    Math::Add ((i32, i32)) -> i32;
}

// This should work exactly as before the fix
#[effectful]
fn traditional_function(x: i32, y: i32) -> String {
    let _: () = perform!(Console::Print("Computing...".to_string()));
    let result: i32 = perform!(Math::Add((x, y)));
    let _: () = perform!(Console::Print(format!("Result: {result}")));
    format!("Answer: {result}")
}

struct TraditionalHandler;

impl Handler<Op> for TraditionalHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[TRADITIONAL] {msg}");
                Box::new(())
            }
            Op::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[TRADITIONAL] {a} + {b} = {result}");
                Box::new(result)
            }
        }
    }
}

fn main() {
    println!("Testing backwards compatibility...");

    let result = traditional_function(15, 25)
        .handle(TraditionalHandler)
        .run();

    println!("Final result: {result}");

    // Verify type is still Effectful<String, Op>
    fn type_check() {
        let _fn: fn(i32, i32) -> Effectful<String, Op> = traditional_function;
        println!("✅ Type is still Effectful<String, Op>");
    }

    type_check();

    println!("✅ Success! Backwards compatibility maintained");
}
