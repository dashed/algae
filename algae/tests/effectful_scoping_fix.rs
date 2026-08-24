#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: argument parsing for the `#[effectful]` attribute.
//!
//! Both spellings have to keep working, and each has to pick the matching root
//! type:
//! - `#[effectful]` defaults to the `Op` root.
//! - `#[effectful(root = CustomOp)]` uses the named root.

use algae::prelude::*;
use std::sync::{Arc, Mutex};

effect! {
    Console::Print (String) -> ();
}

#[effectful] // Uses default Op type
fn default_function() -> String {
    let _: () = perform!(Console::Print("Default function".to_string()));
    "Default success".to_string()
}

effect! {
    root CustomOp;
    Logger::Info (String) -> ();
}

#[effectful(root = CustomOp)] // Uses custom root type
fn custom_function() -> String {
    let _: () = perform!(Logger::Info("Custom function".to_string()));
    "Custom success".to_string()
}

struct DefaultHandler {
    printed: Arc<Mutex<Vec<String>>>,
}

impl Handler<Op> for DefaultHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                self.printed.lock().unwrap().push(msg.clone());
                Box::new(())
            }
        }
    }
}

struct CustomHandler {
    logged: Arc<Mutex<Vec<String>>>,
}

impl Handler<CustomOp> for CustomHandler {
    fn handle(&mut self, op: &CustomOp) -> Box<dyn std::any::Any + Send> {
        match op {
            CustomOp::Logger(Logger::Info(msg)) => {
                self.logged.lock().unwrap().push(msg.clone());
                Box::new(())
            }
        }
    }
}

#[test]
fn effectful_without_arguments_uses_the_default_root() {
    let printed = Arc::new(Mutex::new(Vec::new()));

    let result = default_function()
        .handle(DefaultHandler {
            printed: Arc::clone(&printed),
        })
        .run();

    assert_eq!(result, "Default success");
    assert_eq!(printed.lock().unwrap().as_slice(), ["Default function"]);
}

#[test]
fn effectful_with_root_argument_uses_the_named_root() {
    let logged = Arc::new(Mutex::new(Vec::new()));

    let result = custom_function()
        .handle(CustomHandler {
            logged: Arc::clone(&logged),
        })
        .run();

    assert_eq!(result, "Custom success");
    assert_eq!(logged.lock().unwrap().as_slice(), ["Custom function"]);
}

/// Each function must be generated with the root type it asked for. Coercing to
/// an explicit `fn` pointer fails to compile if either signature drifts.
#[test]
fn generated_signatures_carry_the_expected_root_type() {
    let _default: fn() -> Effectful<String, Op> = default_function;
    let _custom: fn() -> Effectful<String, CustomOp> = custom_function;
}
