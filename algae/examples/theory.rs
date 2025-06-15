#![feature(coroutines, coroutine_trait, yield_expr)]
//! This example demonstrates the mapping between algebraic effects theory
//! and the algae implementation, as described in the README.

use algae::prelude::*;

// Effect Signature: Declares operations and their types
effect! {
    // Mathematical operation effects
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;

    // State effects
    Counter::Get -> i32;
    Counter::Set (i32) -> ();
    Counter::Increment -> ();
}

// Handler: Provides interpretation for operations
struct MathHandler;

impl Handler<Op> for MathHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Math(Math::Add((a, b))) => Box::new(a + b),
            Op::Math(Math::Multiply((a, b))) => Box::new(a * b),
            _ => panic!("MathHandler cannot handle non-Math operations"),
        }
    }
}

struct CounterHandler {
    count: i32,
}

impl CounterHandler {
    fn new(initial: i32) -> Self {
        Self { count: initial }
    }
}

impl Handler<Op> for CounterHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Counter(Counter::Get) => Box::new(self.count),
            Op::Counter(Counter::Set(value)) => {
                self.count = *value;
                Box::new(())
            }
            Op::Counter(Counter::Increment) => {
                self.count += 1;
                Box::new(())
            }
            _ => panic!("CounterHandler cannot handle non-Counter operations"),
        }
    }
}

// Combined handler for multiple effect families
struct CombinedHandler {
    math: MathHandler,
    counter: CounterHandler,
}

impl CombinedHandler {
    fn new(initial_count: i32) -> Self {
        Self {
            math: MathHandler,
            counter: CounterHandler::new(initial_count),
        }
    }
}

impl Handler<Op> for CombinedHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Math(_) => self.math.handle(op),
            Op::Counter(_) => self.counter.handle(op),
        }
    }
}

// Effectful computation that uses multiple effect families
#[effectful]
fn complex_computation(a: i32, b: i32) -> i32 {
    // Effect Operation: Invokes an effect operation
    let sum: i32 = perform!(Math::Add((a, b)));
    println!("Sum: {sum}");

    // Store the result in counter
    let _: () = perform!(Counter::Set(sum));

    // Increment the counter
    let _: () = perform!(Counter::Increment);

    // Get the final value
    let counter_value: i32 = perform!(Counter::Get);
    println!("Counter after increment: {counter_value}");

    // Multiply by 2
    let result: i32 = perform!(Math::Multiply((counter_value, 2)));
    println!("Final result: {result}");

    result
}

// Demonstrate theoretical concepts
fn demonstrate_theory() {
    println!("=== Theoretical Foundations Demo ===\n");

    // 1. Effect Signature → effect! macro
    println!("1. Effect Signatures defined with effect! macro");
    println!("   - Math::Add ((i32, i32)) -> i32");
    println!("   - Counter::Get -> i32");
    println!("   - Counter::Set (i32) -> ()");

    // 2. Effect Operation → perform!(Operation)
    println!("\n2. Effect Operations invoked with perform! macro");

    // 3. Handler → Handler<Op> trait
    println!("\n3. Handlers implement Handler<Op> trait");

    // 4. Handled Computation → Effectful<R, Op>
    println!("\n4. Handled Computation returns Effectful<R, Op>");

    // 5. Handler Installation → .handle(h).run()
    println!("\n5. Handler Installation with .handle(handler).run()");

    println!("\n--- Running the computation ---");

    // This creates an Effectful<i32, Op> (Handled Computation)
    let computation = complex_computation(10, 5);

    // Handler Installation: applies handler to computation
    let result = computation
        .handle(CombinedHandler::new(0)) // Install the handler
        .run(); // Execute the computation

    println!("\nFinal computation result: {result}");

    // Demonstrate different handlers
    println!("\n--- Same computation, different handler ---");
    let result2 = complex_computation(3, 7)
        .handle(CombinedHandler::new(100)) // Different initial state
        .run();

    println!("With different initial state: {result2}");
}

// Demonstrate algebraic laws
#[effectful]
fn demonstrate_associativity() -> i32 {
    // (a + b) * c
    let a = 2;
    let b = 3;
    let c = 4;

    let sum: i32 = perform!(Math::Add((a, b)));
    perform!(Math::Multiply((sum, c)))
}

#[effectful]
fn demonstrate_distributivity_left(a: i32, b: i32, c: i32) -> i32 {
    // a * (b + c)
    let sum: i32 = perform!(Math::Add((b, c)));
    perform!(Math::Multiply((a, sum)))
}

#[effectful]
fn demonstrate_distributivity_right(a: i32, b: i32, c: i32) -> i32 {
    // a * b + a * c
    let product1: i32 = perform!(Math::Multiply((a, b)));
    let product2: i32 = perform!(Math::Multiply((a, c)));
    perform!(Math::Add((product1, product2)))
}

fn demonstrate_algebraic_laws() {
    println!("\n=== Algebraic Laws Demo ===\n");

    // Demonstrate distributivity: a * (b + c) = a * b + a * c
    let left = demonstrate_distributivity_left(2, 3, 4)
        .handle(MathHandler)
        .run();

    let right = demonstrate_distributivity_right(2, 3, 4)
        .handle(MathHandler)
        .run();

    println!("Distributivity law: 2 * (3 + 4) = 2 * 3 + 2 * 4");
    println!("Left side:  {left} (should be 14)");
    println!("Right side: {right} (should be 14)");
    let holds = left == right;
    println!("Law holds: {holds}");

    // Demonstrate associativity with different computations
    let assoc_result = demonstrate_associativity().handle(MathHandler).run();

    println!("\nAssociativity: (2 + 3) * 4 = {assoc_result}");
}

fn main() {
    demonstrate_theory();
    demonstrate_algebraic_laws();

    println!("\n=== Theory Example Completed ===");
    println!("This example shows how algae implements the theoretical");
    println!("foundations of algebraic effects and handlers.");
}
