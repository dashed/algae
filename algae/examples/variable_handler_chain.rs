//! Example demonstrating variable-length handler chains with zero-panic execution.
//!
//! This example shows how to:
//! - Chain multiple handlers together
//! - Mix Handler and PartialHandler types
//! - Use run_checked for panic-free execution
//! - Handle unhandled operations gracefully

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define our effects
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;

    File::Read (String) -> Result<String, String>;
    File::Write ((String, String)) -> Result<(), String>;

    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
    Logger::GetCount -> usize;
}

// Console handler - only handles console operations
struct ConsoleHandler;

impl PartialHandler<Op> for ConsoleHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[CONSOLE] {msg}");
                Some(Box::new(()))
            }
            Op::Console(Console::ReadLine) => {
                println!("[CONSOLE] Simulating user input...");
                Some(Box::new("user input".to_string()))
            }
            _ => None, // Decline non-console operations
        }
    }
}

// File handler - only handles file operations
struct FileHandler {
    files: std::collections::HashMap<String, String>,
}

impl FileHandler {
    fn new() -> Self {
        let mut files = std::collections::HashMap::new();
        files.insert("config.txt".to_string(), "debug=true".to_string());
        files.insert("data.txt".to_string(), "Hello, World!".to_string());
        Self { files }
    }
}

impl PartialHandler<Op> for FileHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::File(File::Read(path)) => {
                let result = self
                    .files
                    .get(path)
                    .cloned()
                    .ok_or_else(|| format!("File not found: {path}"));
                Some(Box::new(result))
            }
            Op::File(File::Write((path, content))) => {
                self.files.insert(path.clone(), content.clone());
                Some(Box::new(Ok::<(), String>(())))
            }
            _ => None, // Decline non-file operations
        }
    }
}

// Logger handler - handles logging operations with state
struct LoggerHandler {
    logs: Vec<(String, String)>,
}

impl LoggerHandler {
    fn new() -> Self {
        Self { logs: Vec::new() }
    }
}

impl PartialHandler<Op> for LoggerHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Logger(Logger::Info(msg)) => {
                println!("[INFO] {msg}");
                self.logs.push(("INFO".to_string(), msg.clone()));
                Some(Box::new(()))
            }
            Op::Logger(Logger::Error(msg)) => {
                eprintln!("[ERROR] {msg}");
                self.logs.push(("ERROR".to_string(), msg.clone()));
                Some(Box::new(()))
            }
            Op::Logger(Logger::GetCount) => Some(Box::new(self.logs.len())),
            _ => None, // Decline non-logger operations
        }
    }
}

// Example 1: Basic three-handler chain
#[effectful]
fn three_handler_example() -> String {
    let _: () = perform!(Logger::Info("Starting three-handler example".to_string()));
    let _: () = perform!(Console::Print("Reading configuration...".to_string()));

    let config: Result<String, String> = perform!(File::Read("config.txt".to_string()));
    match config {
        Ok(content) => {
            let _: () = perform!(Logger::Info(format!("Config loaded: {content}")));
            content
        }
        Err(err) => {
            let _: () = perform!(Logger::Error(format!("Failed to load config: {err}")));
            "default config".to_string()
        }
    }
}

// Example 2: Interactive application with all effects
#[effectful]
fn interactive_app() -> Result<String, String> {
    let _: () = perform!(Logger::Info("Interactive app started".to_string()));
    let _: () = perform!(Console::Print("Welcome! What's your name?".to_string()));

    let name: String = perform!(Console::ReadLine);
    let _: () = perform!(Logger::Info(format!("User entered name: {name}")));

    let _: () = perform!(Console::Print(
        "What file would you like to read?".to_string()
    ));
    let filename = "data.txt"; // Simulating user input

    let content: Result<String, String> = perform!(File::Read(filename.to_string()));
    match content {
        Ok(data) => {
            let _: () = perform!(Logger::Info(format!("Successfully read file: {filename}")));
            let _: () = perform!(Console::Print(format!("File contents: {data}")));

            // Save a personalized greeting
            let greeting = format!("Hello, {name}! {data}");
            let _: Result<(), String> = perform!(File::Write((
                format!("{name}_greeting.txt"),
                greeting.clone()
            )));

            Ok(greeting)
        }
        Err(err) => {
            let _: () = perform!(Logger::Error(format!("Failed to read {filename}: {err}")));
            Err(err)
        }
    }
}

fn main() {
    println!("=== Example 1: Three-Handler Chain ===\n");

    // Method 1: Using handle_all with a vector
    let handlers: Vec<Box<dyn PartialHandler<Op> + Send>> = vec![
        Box::new(ConsoleHandler),
        Box::new(FileHandler::new()),
        Box::new(LoggerHandler::new()),
    ];

    match three_handler_example().handle_all(handlers).run_checked() {
        Ok(result) => println!("\nResult: {result}"),
        Err(UnhandledOp(op)) => eprintln!("\nError: Unhandled operation: {op:?}"),
    }

    println!("\n=== Example 2: Interactive Application ===\n");

    // Method 2: Building VecHandler manually
    let mut vec_handler = VecHandler::new();
    vec_handler.push(ConsoleHandler);
    vec_handler.push(FileHandler::new());
    vec_handler.push(LoggerHandler::new());

    match interactive_app().run_checked(vec_handler) {
        Ok(Ok(result)) => println!("\nSuccess: {result}"),
        Ok(Err(err)) => println!("\nApplication error: {err}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }

    println!("\n=== Example 3: Missing Handler Demonstration ===\n");

    // Only provide console and file handlers, but not logger
    let mut partial_handlers = VecHandler::new();
    partial_handlers.push(ConsoleHandler);
    partial_handlers.push(FileHandler::new());
    // Note: LoggerHandler is missing!

    match interactive_app().run_checked(partial_handlers) {
        Ok(_) => println!("This shouldn't happen - we're missing the logger!"),
        Err(UnhandledOp(op)) => {
            println!("As expected, got unhandled operation: {op:?}");
            println!("This demonstrates safe error handling without panics.");
        }
    }

    println!("\n=== Example 4: Total Handler Wrapping ===\n");

    // Example with a total handler that must handle everything
    struct TotalAppHandler {
        console: ConsoleHandler,
        file: FileHandler,
        logger: LoggerHandler,
    }

    impl Handler<Op> for TotalAppHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            // Delegate to appropriate partial handler
            match op {
                Op::Console(_) => self.console.maybe_handle(op).unwrap(),
                Op::File(_) => self.file.maybe_handle(op).unwrap(),
                Op::Logger(_) => self.logger.maybe_handle(op).unwrap(),
            }
        }
    }

    let total_handler = TotalAppHandler {
        console: ConsoleHandler,
        file: FileHandler::new(),
        logger: LoggerHandler::new(),
    };

    // Total handlers can use run() (may panic) or run_checked_with() (safe)
    match interactive_app().run_checked_with(total_handler) {
        Ok(Ok(result)) => println!("\nWith total handler: {result}"),
        Ok(Err(err)) => println!("\nApplication error: {err}"),
        Err(_) => unreachable!("Total handler handles everything"),
    }

    println!("\n=== Summary ===");
    println!("✅ Variable-length handler chains enable modular effect handling");
    println!("✅ PartialHandler allows handlers to decline operations");
    println!("✅ run_checked() provides panic-free execution");
    println!("✅ Both Handler and PartialHandler types are supported");
    println!("✅ Unhandled operations return clear error information");
}
