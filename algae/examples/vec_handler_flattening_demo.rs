//! Demonstrates the two ways to fold one `VecHandler` into another when chaining.
//!
//! `.merge(vec)` splices the handlers of `vec` into the receiving chain, leaving a
//! single flat collection. `.handle(vec)` nests `vec` as one boxed handler instead.
//! Both preserve handler order, so the two forms answer every operation the same
//! way - `merge` just avoids the extra layer of indirection.

#![feature(coroutines, yield_expr)]
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

#[effectful]
fn process_number(n: i32) -> i32 {
    perform!(Stage::Process(n))
}

/// The arithmetic handlers, grouped as a reusable `VecHandler`.
fn arithmetic_handlers() -> VecHandler<Op> {
    let mut handlers = VecHandler::new();
    handlers.push(AddTenHandler);
    handlers.push(MultiplyTwoHandler);
    handlers
}

/// A second group, holding just the squaring handler.
fn square_handlers() -> VecHandler<Op> {
    let mut handlers = VecHandler::new();
    handlers.push(SquareHandler);
    handlers
}

fn main() {
    println!("=== VecHandler merging demo ===\n");

    println!("Test 1: Process 5 (should be handled by AddTenHandler)");
    {
        let result = process_number(5)
            .begin_chain()
            .merge(arithmetic_handlers()) // Splice both arithmetic handlers in
            .merge(square_handlers()) // ...then the squaring handler
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(unhandled) => println!("{unhandled}\n"),
        }
    }

    println!("Test 2: Process 150 (should be handled by MultiplyTwoHandler)");
    {
        let result = process_number(150)
            .begin_chain()
            .merge(arithmetic_handlers())
            .merge(square_handlers())
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(unhandled) => println!("{unhandled}\n"),
        }
    }

    println!("Test 3: Process 4 (AddTenHandler wins - it comes first in the chain)");
    {
        // 4 is below 100, so AddTenHandler claims it before SquareHandler is reached.
        let result = process_number(4)
            .begin_chain()
            .merge(arithmetic_handlers())
            .merge(square_handlers())
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(unhandled) => println!("{unhandled}\n"),
        }
    }

    println!("Test 4: Demonstrating handler order matters");
    {
        // Reversing the two groups gives SquareHandler priority for even numbers.
        let result = process_number(4)
            .begin_chain()
            .merge(square_handlers()) // SquareHandler first
            .merge(arithmetic_handlers()) // Other handlers second
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n} (SquareHandler took precedence)\n"),
            Err(unhandled) => println!("{unhandled}\n"),
        }
    }

    println!("Test 5: `.handle(vec)` nests instead of flattening - same answers");
    {
        // Passing a VecHandler to `.handle()` stores it as a single boxed handler.
        // The chain is one level deeper, but the operations still reach the same
        // handlers in the same order, so Test 4's result is reproduced exactly.
        let result = process_number(4)
            .begin_chain()
            .handle(square_handlers())
            .handle(arithmetic_handlers())
            .run_checked();

        match result {
            Ok(n) => println!("Result: {n} (identical to Test 4)\n"),
            Err(unhandled) => println!("{unhandled}\n"),
        }
    }

    println!("Test 6: an operation nobody handles");
    {
        // An empty chain declines everything. `run_checked` hands the operation
        // back instead of panicking, which is what `.run()` would do here.
        let result = process_number(7).begin_chain().run_checked();

        match result {
            Ok(n) => println!("Result: {n}\n"),
            Err(unhandled) => println!("As expected: {unhandled}\n"),
        }
    }

    println!("Merging keeps the handler collection flat;");
    println!("nesting via `.handle()` is equivalent, just one indirection deeper.");
}
