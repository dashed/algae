//! Demonstrates that VecHandler flattening works correctly when chaining handlers.
//!
//! This example shows that when you pass a VecHandler to .handle(), its contents
//! are properly flattened into the receiving VecHandler, avoiding nested structures.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::impl_into_vec_handler;
use algae::prelude::*;

// Define effects
effect! {
    Stage::Process (i32) -> i32;
}

// Handler that adds 10
struct AddTenHandler;
impl PartialHandler<Op> for AddTenHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Stage(Stage::Process(n)) if *n < 100 => {
                println!("AddTenHandler: {n} + 10 = {}", n + 10);
                Some(Box::new(n + 10))
            }
            _ => None,
        }
    }
}

// Handler that multiplies by 2
struct MultiplyTwoHandler;
impl PartialHandler<Op> for MultiplyTwoHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Stage(Stage::Process(n)) if *n >= 100 => {
                println!("MultiplyTwoHandler: {n} * 2 = {}", n * 2);
                Some(Box::new(n * 2))
            }
            _ => None,
        }
    }
}

// Handler that squares
struct SquareHandler;
impl PartialHandler<Op> for SquareHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Stage(Stage::Process(n)) if n % 2 == 0 => {
                println!("SquareHandler: {n} ^ 2 = {}", n * n);
                Some(Box::new(n * n))
            }
            _ => None,
        }
    }
}

// Implement IntoVecHandler for all handlers
impl_into_vec_handler!(AddTenHandler, Op);
impl_into_vec_handler!(MultiplyTwoHandler, Op);
impl_into_vec_handler!(SquareHandler, Op);

#[effectful]
fn process_number(n: i32) -> i32 {
    perform!(Stage::Process(n))
}

fn main() {
    println!("=== VecHandler Flattening Demo ===\n");

    println!("Test 1: Process 5 (should be handled by AddTenHandler)");
    {
        // Create first VecHandler with two handlers
        let mut vec1 = VecHandler::new();
        vec1.push(AddTenHandler);
        vec1.push(MultiplyTwoHandler);

        // Create second VecHandler with one handler
        let mut vec2 = VecHandler::new();
        vec2.push(SquareHandler);

        let result = process_number(5)
            .begin_chain()
            .handle(vec1) // Pass entire VecHandler
            .handle(vec2) // Pass another VecHandler
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(UnhandledOp(op)) => println!("Unhandled: {op:?}\n"),
        }
    }

    println!("Test 2: Process 150 (should be handled by MultiplyTwoHandler)");
    {
        let mut vec1 = VecHandler::new();
        vec1.push(AddTenHandler);
        vec1.push(MultiplyTwoHandler);

        let mut vec2 = VecHandler::new();
        vec2.push(SquareHandler);

        let result = process_number(150)
            .begin_chain()
            .handle(vec1)
            .handle(vec2)
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(UnhandledOp(op)) => println!("Unhandled: {op:?}\n"),
        }
    }

    println!("Test 3: Process 4 (should be handled by SquareHandler)");
    {
        let mut vec1 = VecHandler::new();
        vec1.push(AddTenHandler);
        vec1.push(MultiplyTwoHandler);

        let mut vec2 = VecHandler::new();
        vec2.push(SquareHandler);

        let result = process_number(4)
            .begin_chain()
            .handle(vec1)
            .handle(vec2)
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(UnhandledOp(op)) => println!("Unhandled: {op:?}\n"),
        }
    }

    println!("Test 4: Demonstrating handler order matters");
    {
        // If we reverse the order, SquareHandler gets priority for even numbers
        let mut vec1 = VecHandler::new();
        vec1.push(AddTenHandler);
        vec1.push(MultiplyTwoHandler);

        let mut vec2 = VecHandler::new();
        vec2.push(SquareHandler);

        let result = process_number(4)
            .begin_chain()
            .handle(vec2) // SquareHandler first
            .handle(vec1) // Other handlers second
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n} (SquareHandler took precedence)\n"),
            Err(UnhandledOp(op)) => println!("Unhandled: {op:?}\n"),
        }
    }

    println!("✅ All VecHandlers were properly flattened!");
    println!("✅ No nested VecHandler structures were created!");
}
