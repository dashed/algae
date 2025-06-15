//! Demonstrating custom root enum names in effect! macro
//!
//! This example shows how to use the new `root EnumName;` syntax to avoid
//! conflicts when using multiple effect! macros in the same module.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// ============================================================================
// ✅ SOLUTION: Custom root enum names
// ============================================================================

// First effect declaration with custom root name
effect! {
    root ConsoleOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
    Console::Clear -> ();
}

// Second effect declaration with different custom root name
effect! {
    root MathOp;
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
    Math::Divide ((i32, i32)) -> Result<i32, String>;
}

// Third effect declaration with another custom root name
effect! {
    root FileOp;
    File::Read (String) -> Result<String, String>;
    File::Write ((String, String)) -> Result<(), String>;
}

// ============================================================================
// Individual effectful functions for each root type
// ============================================================================

// This function uses ConsoleOp
fn console_demo() -> algae::Effectful<String, ConsoleOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Clear screen
            {
                let effect = algae::Effect::new(Console::Clear.into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Print prompt
            {
                let effect =
                    algae::Effect::new(Console::Print("Enter your name:".to_string()).into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Read input
            let name: String = {
                let effect = algae::Effect::new(Console::ReadLine.into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<String>()
            };

            format!("Hello, {}!", name)
        },
    )
}

// This function uses MathOp
fn math_demo(a: i32, b: i32) -> algae::Effectful<i32, MathOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Add numbers
            let sum: i32 = {
                let effect = algae::Effect::new(Math::Add((a, b)).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<i32>()
            };

            // Multiply by 2
            let doubled: i32 = {
                let effect = algae::Effect::new(Math::Multiply((sum, 2)).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<i32>()
            };

            doubled
        },
    )
}

// This function uses FileOp
fn file_demo(filename: String) -> algae::Effectful<String, FileOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Try to read file
            let content: Result<String, String> = {
                let effect = algae::Effect::new(File::Read(filename.clone()).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<Result<String, String>>()
            };

            match content {
                Ok(data) => data,
                Err(error) => {
                    // Write error to error log
                    let _: Result<(), String> = {
                        let effect = algae::Effect::new(
                            File::Write((
                                "error.log".to_string(),
                                format!("Failed to read {}: {}", filename, error),
                            ))
                            .into(),
                        );
                        let reply_opt = yield effect;
                        reply_opt.unwrap().take::<Result<(), String>>()
                    };

                    "default content".to_string()
                }
            }
        },
    )
}

// ============================================================================
// Handlers for each root type
// ============================================================================

struct ConsoleHandler {
    responses: Vec<String>,
    index: std::cell::RefCell<usize>,
}

impl ConsoleHandler {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: std::cell::RefCell::new(0),
        }
    }
}

impl algae::Handler<ConsoleOp> for ConsoleHandler {
    fn handle(&mut self, op: &ConsoleOp) -> Box<dyn std::any::Any + Send> {
        match op {
            ConsoleOp::Console(Console::Print(msg)) => {
                println!("[CONSOLE] {}", msg);
                Box::new(())
            }
            ConsoleOp::Console(Console::ReadLine) => {
                let mut index = self.index.borrow_mut();
                let response = self
                    .responses
                    .get(*index)
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                *index += 1;
                println!("[CONSOLE] <input: {}>", response);
                Box::new(response)
            }
            ConsoleOp::Console(Console::Clear) => {
                println!("[CONSOLE] <screen cleared>");
                Box::new(())
            }
        }
    }
}

struct MathHandler;

impl algae::Handler<MathOp> for MathHandler {
    fn handle(&mut self, op: &MathOp) -> Box<dyn std::any::Any + Send> {
        match op {
            MathOp::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[MATH] {} + {} = {}", a, b, result);
                Box::new(result)
            }
            MathOp::Math(Math::Multiply((a, b))) => {
                let result = a * b;
                println!("[MATH] {} * {} = {}", a, b, result);
                Box::new(result)
            }
            MathOp::Math(Math::Divide((a, b))) => {
                if *b == 0 {
                    Box::new(Err::<i32, String>("Division by zero".to_string()))
                } else {
                    let result = a / b;
                    println!("[MATH] {} / {} = {}", a, b, result);
                    Box::new(Ok::<i32, String>(result))
                }
            }
        }
    }
}

struct FileHandler {
    files: std::collections::HashMap<String, String>,
}

impl FileHandler {
    fn new() -> Self {
        let mut files = std::collections::HashMap::new();
        files.insert("test.txt".to_string(), "Hello, world!".to_string());
        files.insert("data.txt".to_string(), "Some important data".to_string());

        Self { files }
    }
}

impl algae::Handler<FileOp> for FileHandler {
    fn handle(&mut self, op: &FileOp) -> Box<dyn std::any::Any + Send> {
        match op {
            FileOp::File(File::Read(path)) => {
                if let Some(content) = self.files.get(path) {
                    println!("[FILE] Reading {}: {}", path, content);
                    Box::new(Ok::<String, String>(content.clone()))
                } else {
                    println!("[FILE] File not found: {}", path);
                    Box::new(Err::<String, String>(format!("File not found: {}", path)))
                }
            }
            FileOp::File(File::Write((path, content))) => {
                self.files.insert(path.clone(), content.clone());
                println!("[FILE] Writing to {}: {}", path, content);
                Box::new(Ok::<(), String>(()))
            }
        }
    }
}

// ============================================================================
// Using combine_roots! to create a unified interface
// ============================================================================

// Combine all root enums into one for easier handling
algae::combine_roots!(pub UnifiedOp = ConsoleOp, MathOp, FileOp);

struct UnifiedHandler {
    console: ConsoleHandler,
    math: MathHandler,
    file: FileHandler,
}

impl UnifiedHandler {
    fn new() -> Self {
        Self {
            console: ConsoleHandler::new(vec!["Alice".to_string(), "Bob".to_string()]),
            math: MathHandler,
            file: FileHandler::new(),
        }
    }
}

impl algae::Handler<UnifiedOp> for UnifiedHandler {
    fn handle(&mut self, op: &UnifiedOp) -> Box<dyn std::any::Any + Send> {
        match op {
            UnifiedOp::ConsoleOp(console_op) => self.console.handle(console_op),
            UnifiedOp::MathOp(math_op) => self.math.handle(math_op),
            UnifiedOp::FileOp(file_op) => self.file.handle(file_op),
        }
    }
}

// ============================================================================
// Conversion functions to work with unified handler
// ============================================================================

fn console_demo_unified() -> algae::Effectful<String, UnifiedOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Clear screen
            {
                let effect = algae::Effect::new(UnifiedOp::ConsoleOp(Console::Clear.into()));
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Print prompt
            {
                let effect = algae::Effect::new(UnifiedOp::ConsoleOp(
                    Console::Print("Enter your name:".to_string()).into(),
                ));
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Read input
            let name: String = {
                let effect = algae::Effect::new(UnifiedOp::ConsoleOp(Console::ReadLine.into()));
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<String>()
            };

            format!("Hello, {}!", name)
        },
    )
}

// ============================================================================
// DEMONSTRATION
// ============================================================================

fn main() {
    println!("=== Custom Root Effects Demo ===\n");

    println!("1. Individual handlers with custom root names:");

    // Console demo
    let console_result = console_demo()
        .handle(ConsoleHandler::new(vec!["Alice".to_string()]))
        .run();
    println!("Console result: {}\n", console_result);

    // Math demo
    let math_result = math_demo(10, 5).handle(MathHandler).run();
    println!("Math result: {}\n", math_result);

    // File demo
    let file_result = file_demo("test.txt".to_string())
        .handle(FileHandler::new())
        .run();
    println!("File result: {}\n", file_result);

    println!("2. Unified handler using combine_roots! macro:");

    // Unified demo
    let unified_result = console_demo_unified().handle(UnifiedHandler::new()).run();
    println!("Unified result: {}\n", unified_result);

    println!("=== Key Benefits ===");
    println!("✅ Multiple effect! macros in same module without conflicts");
    println!("✅ Clear separation of concerns with named root enums");
    println!("✅ Easy unification with combine_roots! macro");
    println!("✅ Backward compatibility - existing code still works");
}
