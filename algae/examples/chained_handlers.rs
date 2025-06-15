//! Example demonstrating chained handler syntax.
//!
//! This example shows how to chain multiple handlers using:
//! - `.handle_all()` followed by `.handle()` calls
//! - Building up handler chains incrementally

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define our effects
effect! {
    Console::Print (String) -> ();
    File::Read (String) -> Result<String, String>;
    Logger::Info (String) -> ();
}

// Console handler
struct ConsoleHandler;

impl PartialHandler<Op> for ConsoleHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[CONSOLE] {msg}");
                Some(Box::new(()))
            }
            _ => None,
        }
    }
}

// File handler
struct FileHandler;

impl PartialHandler<Op> for FileHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::File(File::Read(path)) => {
                if path == "config.txt" {
                    Some(Box::new(Ok::<String, String>("debug=true".to_string())))
                } else {
                    Some(Box::new(Err::<String, String>(format!(
                        "File not found: {path}"
                    ))))
                }
            }
            _ => None,
        }
    }
}

// Logger handler
struct LoggerHandler;

impl PartialHandler<Op> for LoggerHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Logger(Logger::Info(msg)) => {
                println!("[INFO] {msg}");
                Some(Box::new(()))
            }
            _ => None,
        }
    }
}

#[effectful]
fn chained_computation() -> String {
    let _: () = perform!(Logger::Info("Starting chained computation".to_string()));
    let _: () = perform!(Console::Print("Loading configuration...".to_string()));
    let config: Result<String, String> = perform!(File::Read("config.txt".to_string()));
    match config {
        Ok(content) => {
            let _: () = perform!(Logger::Info(format!("Config loaded: {content}")));
            content
        }
        Err(err) => {
            let _: () = perform!(Logger::Info(format!("Failed to load config: {err}")));
            "default".to_string()
        }
    }
}

fn main() {
    println!("=== Example 1: Starting with begin_chain ===\n");

    // Start with begin_chain() and chain handlers
    let result = chained_computation()
        .begin_chain()
        .handle(ConsoleHandler) // Add first handler
        .handle(FileHandler) // Add second handler
        .handle(LoggerHandler) // Add third handler
        .run_checked();

    match result {
        Ok(value) => println!("\nComputation succeeded with: {value}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }

    println!("\n=== Example 2: Starting with one handler ===\n");

    // Start with one handler and chain more
    let result = chained_computation()
        .handle_all([ConsoleHandler]) // Start with one
        .handle(FileHandler) // Add second
        .handle(LoggerHandler) // Add third
        .run_checked();

    match result {
        Ok(value) => println!("\nComputation succeeded with: {value}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }

    println!("\n=== Example 3: Mixing Handler and PartialHandler ===\n");

    // Total handler that handles everything
    struct TotalHandler;
    impl Handler<Op> for TotalHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            match op {
                Op::Console(Console::Print(msg)) => {
                    println!("[TOTAL] Console: {msg}");
                    Box::new(())
                }
                Op::File(File::Read(_)) => {
                    Box::new(Ok::<String, String>("from total handler".to_string()))
                }
                Op::Logger(Logger::Info(msg)) => {
                    println!("[TOTAL] Logger: {msg}");
                    Box::new(())
                }
            }
        }
    }

    // Can mix total handlers using handle_total
    let result = chained_computation()
        .handle_all([ConsoleHandler])
        .handle(FileHandler)
        .handle_total(TotalHandler) // This will override the previous handlers
        .run_checked();

    match result {
        Ok(value) => println!("\nComputation succeeded with: {value}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }

    println!("\n=== Example 4: Building handler chain dynamically ===\n");

    // Build handler chain based on runtime conditions
    let use_file_handler = true;
    let use_logger = true;

    let mut handled = chained_computation().begin_chain().handle(ConsoleHandler);

    if use_file_handler {
        handled = handled.handle(FileHandler);
    }

    if use_logger {
        handled = handled.handle(LoggerHandler);
    }

    match handled.run_checked() {
        Ok(value) => println!("\nComputation succeeded with: {value}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }
}
