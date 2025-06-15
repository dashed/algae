//! Test that #[effectful] attribute works correctly with custom root types
//!
//! This example tests the bug fix where #[effectful] was hardcoded to use `Op`
//! but now supports custom root types via #[effectful(root = CustomOp)].

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define effects with custom root
effect! {
    root CustomOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
    Math::Add ((i32, i32)) -> i32;
}

// Test #[effectful] with custom root - this should work now
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

// Test that regular #[effectful] still defaults to Op
effect! {
    SimpleOps::GetValue -> i32;
}

#[effectful] // Should default to Op
fn simple_function() -> i32 {
    perform!(SimpleOps::GetValue)
}

// Handlers
struct CustomHandler {
    input: String,
}

impl CustomHandler {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
        }
    }
}

impl Handler<CustomOp> for CustomHandler {
    fn handle(&mut self, op: &CustomOp) -> Box<dyn std::any::Any + Send> {
        match op {
            CustomOp::Console(Console::Print(msg)) => {
                println!("[CUSTOM CONSOLE] {msg}");
                Box::new(())
            }
            CustomOp::Console(Console::ReadLine) => {
                println!("[CUSTOM CONSOLE] <reading: {}>", self.input);
                Box::new(self.input.clone())
            }
            CustomOp::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[CUSTOM MATH] {a} + {b} = {result}");
                Box::new(result)
            }
        }
    }
}

struct SimpleHandler;

impl Handler<Op> for SimpleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::SimpleOps(SimpleOps::GetValue) => {
                println!("[SIMPLE] Getting value: 42");
                Box::new(42)
            }
        }
    }
}

fn main() {
    println!("Testing #[effectful] with custom root types...");

    // Test custom root type
    let custom_result = greet_and_calculate(10, 20)
        .handle(CustomHandler::new("Alice"))
        .run();
    println!("Custom root result: {custom_result}");

    // Test default root type (Op)
    let simple_result = simple_function().handle(SimpleHandler).run();
    println!("Simple result: {simple_result}");

    // Verify types are correct at compile time
    fn type_check() {
        // This function verifies that the types are generated correctly
        let _custom_fn: fn(i32, i32) -> Effectful<String, CustomOp> = greet_and_calculate;
        let _simple_fn: fn() -> Effectful<i32, Op> = simple_function;

        println!("✅ Type checking passed!");
    }

    type_check();

    println!("✅ Success! #[effectful] works with both default and custom root types");
}
