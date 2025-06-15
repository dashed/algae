//! Test that users can manually implement Default for effect types if needed
//!
//! This demonstrates that while we no longer auto-generate Default implementations,
//! users can still add them manually when appropriate.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

effect! {
    SimpleOps::NoPayload -> String;
    SimpleOps::WithPayload (i32) -> String;
}

// Users can manually implement Default if they want to
#[allow(clippy::derivable_impls)] // Intentional manual implementation for demonstration
impl Default for SimpleOps {
    fn default() -> Self {
        SimpleOps::NoPayload
    }
}

impl Default for Op {
    fn default() -> Self {
        Op::SimpleOps(SimpleOps::default())
    }
}

#[effectful]
fn test_function() -> String {
    // Can use Default if manually implemented
    let default_op = SimpleOps::default();
    println!("Default operation: {default_op:?}");

    // Regular usage still works
    let _result1: String = perform!(SimpleOps::NoPayload);
    let _result2: String = perform!(SimpleOps::WithPayload(42));

    "Manual Default works".to_string()
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::SimpleOps(SimpleOps::NoPayload) => Box::new("No payload result".to_string()),
            Op::SimpleOps(SimpleOps::WithPayload(value)) => {
                Box::new(format!("Payload result: {value}"))
            }
        }
    }
}

fn main() {
    println!("Testing manual Default implementation...");

    // Test that Default works when manually implemented
    let default_simple = SimpleOps::default();
    let default_op = Op::default();

    println!("Default SimpleOps: {default_simple:?}");
    println!("Default Op: {default_op:?}");

    let result = test_function().handle(TestHandler).run();
    println!("Result: {result}");

    println!("✅ Success! Manual Default implementations work correctly");
}
