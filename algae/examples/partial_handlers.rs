//! Example demonstrating panic-free, variadic handler design with partial handlers.
//!
//! This example shows how to:
//! - Create partial handlers that only handle specific operations
//! - Compose multiple handlers together
//! - Get Result-based error handling instead of panics
//! - Build modular, testable effect systems

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define our effects
effect! {
    IO::Print (String) -> ();
    IO::ReadLine -> String;

    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
    Math::Divide ((i32, i32)) -> Result<i32, String>;

    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
}

// Partial handler for stdout operations only
struct StdoutHandler;

impl PartialHandler<Op> for StdoutHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::IO(IO::Print(s)) => {
                println!("{s}");
                Some(Box::new(()))
            }
            _ => None, // Decline other operations
        }
    }
}

// Partial handler for stdin operations only
struct StdinHandler;

impl PartialHandler<Op> for StdinHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::IO(IO::ReadLine) => {
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                let line = stdin
                    .lock()
                    .lines()
                    .next()
                    .unwrap_or_else(|| Ok("".to_string()))
                    .unwrap_or_else(|_| "".to_string());
                Some(Box::new(line))
            }
            _ => None,
        }
    }
}

// Partial handler for math operations
struct CalculatorHandler;

impl PartialHandler<Op> for CalculatorHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Math(Math::Add((a, b))) => Some(Box::new(a + b)),
            Op::Math(Math::Multiply((a, b))) => Some(Box::new(a * b)),
            Op::Math(Math::Divide((a, b))) => {
                if *b == 0 {
                    Some(Box::new(Err::<i32, String>("Division by zero".to_string())))
                } else {
                    Some(Box::new(Ok::<i32, String>(a / b)))
                }
            }
            _ => None,
        }
    }
}

// Partial handler for logging (with state)
struct LoggerHandler {
    logs: Vec<(String, String)>, // (level, message)
}

impl LoggerHandler {
    fn new() -> Self {
        Self { logs: Vec::new() }
    }

    #[allow(dead_code)]
    fn print_summary(&self) {
        println!("\n=== Log Summary ===");
        println!("Total logs: {}", self.logs.len());
        for (level, msg) in &self.logs {
            println!("[{level}] {msg}");
        }
    }
}

impl PartialHandler<Op> for LoggerHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Logger(Logger::Info(msg)) => {
                self.logs.push(("INFO".to_string(), msg.clone()));
                println!("[INFO] {msg}");
                Some(Box::new(()))
            }
            Op::Logger(Logger::Error(msg)) => {
                self.logs.push(("ERROR".to_string(), msg.clone()));
                eprintln!("[ERROR] {msg}");
                Some(Box::new(()))
            }
            _ => None,
        }
    }
}

// Example 1: Basic usage with individual handlers
#[effectful]
fn calculator_program() -> i32 {
    let _: () = perform!(IO::Print("Enter two numbers to add:".to_string()));
    let _: () = perform!(Logger::Info("Starting calculation".to_string()));

    // In a real program, you'd read these from stdin
    let a = 10;
    let b = 20;

    let sum: i32 = perform!(Math::Add((a, b)));
    let _: () = perform!(IO::Print(format!("{a} + {b} = {sum}")));

    let product: i32 = perform!(Math::Multiply((sum, 2)));
    let _: () = perform!(IO::Print(format!("{sum} * 2 = {product}")));

    product
}

// Example 2: Program that might have unhandled effects
#[effectful]
fn risky_program() -> Result<i32, String> {
    let _: () = perform!(Logger::Info("Starting risky calculation".to_string()));

    let result: Result<i32, String> = perform!(Math::Divide((10, 0)));
    match result {
        Ok(val) => {
            let _: () = perform!(Logger::Info(format!("Division successful: {val}")));
            Ok(val)
        }
        Err(err) => {
            let _: () = perform!(Logger::Error(format!("Division failed: {err}")));
            Err(err)
        }
    }
}

// Example 3: Interactive program combining multiple effects
#[effectful]
fn interactive_calculator() -> Result<i32, String> {
    let _: () = perform!(IO::Print("=== Interactive Calculator ===".to_string()));
    let _: () = perform!(Logger::Info("Starting interactive session".to_string()));

    let _: () = perform!(IO::Print("Enter first number:".to_string()));
    // Simulating user input for the example
    let x = 15;

    let _: () = perform!(IO::Print("Enter second number:".to_string()));
    let y = 3;

    let _: () = perform!(IO::Print("Enter operation (+, *, /):".to_string()));
    let operation = "*"; // Simulating user input

    let result = match operation {
        "+" => {
            let sum: i32 = perform!(Math::Add((x, y)));
            Ok(sum)
        }
        "*" => {
            let product: i32 = perform!(Math::Multiply((x, y)));
            Ok(product)
        }
        "/" => {
            let div_result: Result<i32, String> = perform!(Math::Divide((x, y)));
            div_result
        }
        _ => {
            let _: () = perform!(Logger::Error("Unknown operation".to_string()));
            Err("Unknown operation".to_string())
        }
    };

    let display_msg = match &result {
        Ok(val) => format!("Result: {val}"),
        Err(err) => format!("Error: {err}"),
    };

    let log_msg = match &result {
        Ok(val) => format!("Calculation completed: {val}"),
        Err(err) => err.clone(),
    };

    let _: () = perform!(IO::Print(display_msg));

    if result.is_ok() {
        let _: () = perform!(Logger::Info(log_msg));
    } else {
        let _: () = perform!(Logger::Error(log_msg));
    }

    result
}

fn main() {
    println!("=== Example 1: Basic Calculator with All Handlers ===\n");

    // Create individual handlers
    let stdout = StdoutHandler;
    let calculator = CalculatorHandler;
    let logger = LoggerHandler::new();

    // Method 1: Using VecHandler manually
    let mut vec_handler = VecHandler::new();
    vec_handler.push(stdout);
    vec_handler.push(calculator);
    vec_handler.push(logger);

    match calculator_program().run_checked(vec_handler) {
        Ok(result) => println!("\nProgram completed successfully with result: {result}"),
        Err(UnhandledOp(op)) => eprintln!("\nError: Unhandled operation: {op:?}"),
    }

    println!("\n=== Example 2: Missing Handler Demonstration ===\n");

    // Only provide math handler, missing logger handler
    let calculator_only = CalculatorHandler;

    match risky_program().run_checked(calculator_only) {
        Ok(_) => println!("This shouldn't happen - we're missing the logger!"),
        Err(UnhandledOp(op)) => {
            println!("As expected, got unhandled operation: {op:?}");
            println!("This is because we didn't provide a Logger handler.");
        }
    }

    println!("\n=== Example 3: Using handle_all for Convenience ===\n");

    // Create handlers with state tracking
    let logger_with_state = LoggerHandler::new();

    // Using handle_all with a vector of boxed handlers
    let handlers: Vec<Box<dyn PartialHandler<Op> + Send>> = vec![
        Box::new(StdoutHandler),
        Box::new(StdinHandler),
        Box::new(CalculatorHandler),
        Box::new(LoggerHandler::new()),
    ];

    match interactive_calculator().handle_all(handlers).run_checked() {
        Ok(result) => {
            println!("\nInteractive calculation completed: {result:?}");
        }
        Err(UnhandledOp(op)) => {
            eprintln!("\nUnhandled operation in interactive mode: {op:?}");
        }
    }

    println!("\n=== Example 4: Stateful Handler ===\n");

    // Run a program with a stateful logger
    let mut vec_handler_stateful = VecHandler::new();
    vec_handler_stateful.push(StdoutHandler);
    vec_handler_stateful.push(CalculatorHandler);
    vec_handler_stateful.push(logger_with_state);

    let result = interactive_calculator().run_checked(vec_handler_stateful);

    match result {
        Ok(_) => {
            // Note: We can't access logger_with_state here because it was moved
            println!("\nProgram with stateful logger completed");
        }
        Err(UnhandledOp(op)) => {
            eprintln!("\nUnhandled: {op:?}");
        }
    }

    println!("\n=== Example 5: Handler Ordering Matters ===\n");

    // Handler that intercepts all math operations and returns 42
    struct InterceptorHandler;
    impl PartialHandler<Op> for InterceptorHandler {
        fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
            match op {
                Op::Math(_) => {
                    println!("[INTERCEPTOR] Intercepting math operation, returning 42");
                    Some(Box::new(42i32))
                }
                _ => None,
            }
        }
    }

    // First interceptor, then real calculator
    let mut vec1 = VecHandler::new();
    vec1.push(StdoutHandler);
    vec1.push(LoggerHandler::new());
    vec1.push(InterceptorHandler);
    vec1.push(CalculatorHandler);

    println!("With interceptor first:");
    let _ = calculator_program().run_checked(vec1);

    // Real calculator first, then interceptor (interceptor never gets called)
    let mut vec2 = VecHandler::new();
    vec2.push(StdoutHandler);
    vec2.push(LoggerHandler::new());
    vec2.push(CalculatorHandler);
    vec2.push(InterceptorHandler);

    println!("\nWith real calculator first:");
    let _ = calculator_program().run_checked(vec2);

    println!("\n=== Summary ===\n");
    println!("Partial handlers provide:");
    println!("- ✅ Composable effect handling");
    println!("- ✅ No panics - Result-based error handling");
    println!("- ✅ Modular handlers that handle only what they know");
    println!("- ✅ Ability to chain multiple handlers");
    println!("- ✅ Clear error messages for unhandled operations");
}
