//! Test that demonstrates argument parsing fix for #[effectful]
//!
//! This example tests that #[effectful] properly parses both:
//! - #[effectful] - defaults to Op type
//! - #[effectful(root = CustomOp)] - uses custom root type

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Test default behavior
effect! {
    Console::Print (String) -> ();
}

#[effectful] // Uses default Op type
fn default_function() -> String {
    let _: () = perform!(Console::Print("Default function".to_string()));
    "Default success".to_string()
}

// Test custom root behavior
effect! {
    root CustomOp;
    Logger::Info (String) -> ();
}

#[effectful(root = CustomOp)] // Uses custom root type
fn custom_function() -> String {
    let _: () = perform!(Logger::Info("Custom function".to_string()));
    "Custom success".to_string()
}

// Handlers
struct DefaultHandler;

impl Handler<Op> for DefaultHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[DEFAULT] {msg}");
                Box::new(())
            }
        }
    }
}

struct CustomHandler;

impl Handler<CustomOp> for CustomHandler {
    fn handle(&mut self, op: &CustomOp) -> Box<dyn std::any::Any + Send> {
        match op {
            CustomOp::Logger(Logger::Info(msg)) => {
                println!("[CUSTOM] {msg}");
                Box::new(())
            }
        }
    }
}

fn main() {
    println!("Testing #[effectful] argument parsing...");

    // Test default behavior
    let default_result = default_function().handle(DefaultHandler).run();
    println!("Default result: {default_result}");

    // Test custom root behavior
    let custom_result = custom_function().handle(CustomHandler).run();
    println!("Custom result: {custom_result}");

    // Verify type signatures are correct
    fn type_check() {
        use algae::Effectful;

        let _default: fn() -> Effectful<String, Op> = default_function;
        let _custom: fn() -> Effectful<String, CustomOp> = custom_function;

        println!("✅ Type signatures are correct!");
    }

    type_check();

    println!("✅ Success! #[effectful] argument parsing works correctly");
}
