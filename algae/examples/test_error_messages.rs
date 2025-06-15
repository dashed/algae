//! Example demonstrating improved error messages for type mismatches

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

effect! {
    Test::GetNumber -> i32;
    Test::GetString -> String;
}

struct BadHandler;

impl Handler<Op> for BadHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Test(Test::GetNumber) => {
                // Wrong! Should return i32, but returning String
                Box::new("42".to_string())
            }
            Op::Test(Test::GetString) => {
                // Wrong! Should return String, but returning i32
                Box::new(42i32)
            }
        }
    }
}

#[effectful]
fn test_number() -> i32 {
    perform!(Test::GetNumber)
}

#[effectful]
fn test_string() -> String {
    perform!(Test::GetString)
}

fn main() {
    println!("This example demonstrates improved error messages for type mismatches.\n");

    println!("1. Trying to get i32 but handler returns String:");
    let result = std::panic::catch_unwind(|| test_number().handle(BadHandler).run());

    if let Err(panic_payload) = result {
        if let Some(msg) = panic_payload.downcast_ref::<String>() {
            println!("Error: {}\n", msg);
        } else if let Some(msg) = panic_payload.downcast_ref::<&str>() {
            println!("Error: {}\n", msg);
        }
    }

    println!("2. Trying to get String but handler returns i32:");
    let result = std::panic::catch_unwind(|| test_string().handle(BadHandler).run());

    if let Err(panic_payload) = result {
        if let Some(msg) = panic_payload.downcast_ref::<String>() {
            println!("Error: {}\n", msg);
        } else if let Some(msg) = panic_payload.downcast_ref::<&str>() {
            println!("Error: {}\n", msg);
        }
    }

    println!(
        "Notice how the error messages now show actual type names instead of useless TypeIds!"
    );
}
