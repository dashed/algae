//! Example demonstrating both explicit and convenient syntax for effectful functions
//! 
//! This example shows that `#[effectful]` and `perform!` are pure convenience macros
//! that generate exactly the same code as the explicit approach.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define our effects
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
}

// ============================================================================
// EXPLICIT APPROACH - Shows exactly what's happening
// ============================================================================

/// Explicit function that returns Effectful<R, Op> - no macros involved
fn calculate_explicit(x: i32, y: i32) -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        // Print start message
        {
            let effect = Effect::new(Console::Print(format!("Calculating {} * ({} + 5)", x, y)).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        // Add 5 to y
        let y_plus_5: i32 = {
            let effect = Effect::new(Math::Add((y, 5)).into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<i32>()
        };
        
        // Multiply x by the result
        let result: i32 = {
            let effect = Effect::new(Math::Multiply((x, y_plus_5)).into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<i32>()
        };
        
        // Print result
        {
            let effect = Effect::new(Console::Print(format!("Result: {}", result)).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        format!("Final answer: {}", result)
    })
}

/// Another explicit function for user interaction
fn greet_user_explicit() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        // Print prompt
        {
            let effect = Effect::new(Console::Print("What's your name?".to_string()).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        // Read input
        let name: String = {
            let effect = Effect::new(Console::ReadLine.into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<String>()
        };
        
        // Print greeting
        {
            let effect = Effect::new(Console::Print(format!("Nice to meet you, {}!", name)).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        format!("User: {}", name)
    })
}

// ============================================================================
// CONVENIENT APPROACH - Same behavior, cleaner syntax
// ============================================================================

/// Convenient function using macros - identical behavior to calculate_explicit
#[effectful]
fn calculate_convenient(x: i32, y: i32) -> String {
    let _: () = perform!(Console::Print(format!("Calculating {} * ({} + 5)", x, y)));
    let y_plus_5: i32 = perform!(Math::Add((y, 5)));
    let result: i32 = perform!(Math::Multiply((x, y_plus_5)));
    let _: () = perform!(Console::Print(format!("Result: {}", result)));
    format!("Final answer: {}", result)
}

/// Convenient function using macros - identical behavior to greet_user_explicit
#[effectful]
fn greet_user_convenient() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    let _: () = perform!(Console::Print(format!("Nice to meet you, {}!", name)));
    format!("User: {}", name)
}

// ============================================================================
// HANDLERS - Same for both approaches
// ============================================================================

struct MockHandler {
    console_input: String,
    console_outputs: Vec<String>,
}

impl MockHandler {
    fn new(input: &str) -> Self {
        Self {
            console_input: input.to_string(),
            console_outputs: Vec::new(),
        }
    }
    
    fn get_outputs(&self) -> &[String] {
        &self.console_outputs
    }
}

impl Handler<Op> for MockHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[MOCK CONSOLE] {}", msg);
                self.console_outputs.push(msg.clone());
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                println!("[MOCK CONSOLE] <reading: {}>", self.console_input);
                Box::new(self.console_input.clone())
            }
            Op::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[MOCK MATH] {} + {} = {}", a, b, result);
                Box::new(result)
            }
            Op::Math(Math::Multiply((a, b))) => {
                let result = a * b;
                println!("[MOCK MATH] {} * {} = {}", a, b, result);
                Box::new(result)
            }
        }
    }
}

// ============================================================================
// DEMONSTRATION
// ============================================================================

fn main() {
    println!("=== Demonstrating Explicit vs Convenient Syntax ===\n");
    
    // Both approaches produce identical results
    println!("1. Math calculation example:");
    
    let result_explicit = calculate_explicit(3, 7)
        .handle(MockHandler::new(""))
        .run();
    println!("Explicit result: {}\n", result_explicit);
    
    let result_convenient = calculate_convenient(3, 7)
        .handle(MockHandler::new(""))
        .run();
    println!("Convenient result: {}\n", result_convenient);
    
    // Verify they're identical
    assert_eq!(result_explicit, result_convenient);
    println!("✅ Both approaches produce identical results!\n");
    
    // User interaction example
    println!("2. User interaction example:");
    
    let user_explicit = greet_user_explicit()
        .handle(MockHandler::new("Alice"))
        .run();
    println!("Explicit result: {}\n", user_explicit);
    
    let user_convenient = greet_user_convenient()
        .handle(MockHandler::new("Alice"))
        .run();
    println!("Convenient result: {}\n", user_convenient);
    
    // Verify they're identical
    assert_eq!(user_explicit, user_convenient);
    println!("✅ Both approaches produce identical results!\n");
    
    // Show type signatures are the same
    println!("3. Type verification:");
    
    // Both functions have the same type
    let _explicit_fn: fn(i32, i32) -> Effectful<String, Op> = calculate_explicit;
    let convenient_fn = calculate_convenient; // This also has type fn(i32, i32) -> Effectful<String, Op>
    
    // We can use them interchangeably
    let functions: Vec<fn(i32, i32) -> Effectful<String, Op>> = vec![
        calculate_explicit,
        convenient_fn,
    ];
    
    println!("✅ Both functions have identical type signatures!");
    println!("✅ Both functions can be stored in the same collection!");
    println!("✅ Both functions produce the same Effectful<String, Op> type!");
    
    println!("\n=== Conclusion ===");
    println!("The #[effectful] macro and perform! are pure convenience:");
    println!("- Same runtime behavior");
    println!("- Same type signatures");
    println!("- Same performance characteristics");
    println!("- Just much cleaner syntax!");
}