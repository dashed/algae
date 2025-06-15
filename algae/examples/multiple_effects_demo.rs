//! Demonstrating multiple effect! macro usage patterns
//!
//! This example shows the different approaches for organizing effects in larger applications:
//! 1. Single effect! with multiple families (traditional approach)
//! 2. Multiple effect! with custom root names
//! 3. Module-based separation for large codebases

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// ============================================================================
// ✅ APPROACH 1: Single effect! macro with multiple families (TRADITIONAL)
// ============================================================================

effect! {
    // Console operations
    Console::Print (String) -> ();
    Console::ReadLine -> String;

    // Math operations
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
    Math::Divide ((i32, i32)) -> Result<i32, String>;

    // File operations
    File::Read (String) -> Result<String, String>;
    File::Write ((String, String)) -> Result<(), String>;

    // Logger operations
    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
    Logger::Debug (String) -> ();

    // Network operations
    Http::Get (String) -> Result<String, String>;
    Http::Post ((String, String)) -> Result<String, String>;
}

#[effectful]
fn comprehensive_demo() -> String {
    let _: () = perform!(Logger::Info("Starting comprehensive demo".to_string()));

    // Console interaction
    let _: () = perform!(Console::Print("Enter your name:".to_string()));
    let name: String = perform!(Console::ReadLine);

    // Math operations
    let x: i32 = 10;
    let y: i32 = 5;
    let sum: i32 = perform!(Math::Add((x, y)));
    let product: i32 = perform!(Math::Multiply((sum, 2)));

    // File operations
    let content = format!("User: {}, Calculation: {}", name, product);
    let _: Result<(), String> = perform!(File::Write(("demo.txt".to_string(), content.clone())));

    // Network request (simulated)
    let _: Result<String, String> = perform!(Http::Get("https://api.example.com/data".to_string()));

    let _: () = perform!(Logger::Info("Demo completed successfully".to_string()));

    format!("Demo result: {} calculated {}", name, product)
}

// ============================================================================
// ✅ APPROACH 2: Multiple effect! with custom root names
// ============================================================================

// Console effects with custom root name
effect! {
    root ConsoleOp;
    ConsoleV2::Print (String) -> ();
    ConsoleV2::ReadLine -> String;
    ConsoleV2::Clear -> ();
}

// Math effects with custom root name
effect! {
    root MathOp;
    MathV2::Add ((i32, i32)) -> i32;
    MathV2::Multiply ((i32, i32)) -> i32;
    MathV2::Divide ((i32, i32)) -> Result<i32, String>;
    MathV2::Power ((i32, u32)) -> i32;
}

// File effects with custom root name
effect! {
    root FileOp;
    FileV2::Read (String) -> Result<String, String>;
    FileV2::Write ((String, String)) -> Result<(), String>;
    FileV2::Delete (String) -> Result<(), String>;
}

// Logger effects with custom root name
effect! {
    root LoggerOp;
    LoggerV2::Info (String) -> ();
    LoggerV2::Error (String) -> ();
    LoggerV2::Debug (String) -> ();
    LoggerV2::GetLogCount -> usize;
}

// Individual effectful functions for each root type
fn console_demo_v2() -> algae::Effectful<String, ConsoleOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Clear screen
            {
                let effect = algae::Effect::new(ConsoleV2::Clear.into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Print welcome message
            {
                let effect = algae::Effect::new(
                    ConsoleV2::Print("Welcome to Console Demo v2!".to_string()).into(),
                );
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Read input
            let name: String = {
                let effect = algae::Effect::new(ConsoleV2::ReadLine.into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<String>()
            };

            format!("Hello from v2, {}!", name)
        },
    )
}

fn math_demo_v2(x: i32, y: i32) -> algae::Effectful<i32, MathOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Add numbers
            let sum: i32 = {
                let effect = algae::Effect::new(MathV2::Add((x, y)).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<i32>()
            };

            // Multiply by 3
            let product: i32 = {
                let effect = algae::Effect::new(MathV2::Multiply((sum, 3)).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<i32>()
            };

            // Calculate power
            let power: i32 = {
                let effect = algae::Effect::new(MathV2::Power((product, 2)).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<i32>()
            };

            power
        },
    )
}

fn file_demo_v2() -> algae::Effectful<String, FileOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Try to read file
            let content: Result<String, String> = {
                let effect = algae::Effect::new(FileV2::Read("config.txt".to_string()).into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<Result<String, String>>()
            };

            match content {
                Ok(data) => data,
                Err(_) => {
                    // Write default config
                    {
                        let effect = algae::Effect::new(
                            FileV2::Write(("config.txt".to_string(), "default config".to_string()))
                                .into(),
                        );
                        let reply_opt = yield effect;
                        let _: Result<(), String> = reply_opt.unwrap().take::<Result<(), String>>();
                    }
                    "default config".to_string()
                }
            }
        },
    )
}

fn logger_demo_v2() -> algae::Effectful<usize, LoggerOp> {
    algae::Effectful::new(
        #[coroutine]
        move |mut _reply: Option<algae::Reply>| {
            // Log info message
            {
                let effect =
                    algae::Effect::new(LoggerV2::Info("Starting logger demo".to_string()).into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Log debug message
            {
                let effect =
                    algae::Effect::new(LoggerV2::Debug("Debug message".to_string()).into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Log error message
            {
                let effect = algae::Effect::new(LoggerV2::Error("Test error".to_string()).into());
                let reply_opt = yield effect;
                let _: () = reply_opt.unwrap().take::<()>();
            }

            // Get log count
            let count: usize = {
                let effect = algae::Effect::new(LoggerV2::GetLogCount.into());
                let reply_opt = yield effect;
                reply_opt.unwrap().take::<usize>()
            };

            count
        },
    )
}

// Individual handlers for each custom root type
struct ConsoleHandler {
    responses: Vec<String>,
    index: usize,
}

impl ConsoleHandler {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: 0,
        }
    }
}

impl algae::Handler<ConsoleOp> for ConsoleHandler {
    fn handle(&mut self, op: &ConsoleOp) -> Box<dyn std::any::Any + Send> {
        match op {
            ConsoleOp::ConsoleV2(ConsoleV2::Print(msg)) => {
                println!("[CUSTOM CONSOLE] {}", msg);
                Box::new(())
            }
            ConsoleOp::ConsoleV2(ConsoleV2::ReadLine) => {
                let response = if self.index < self.responses.len() {
                    self.responses[self.index].clone()
                } else {
                    "default".to_string()
                };
                self.index += 1;
                println!("[CUSTOM CONSOLE] <input: {}>", response);
                Box::new(response)
            }
            ConsoleOp::ConsoleV2(ConsoleV2::Clear) => {
                println!("[CUSTOM CONSOLE] <screen cleared>");
                Box::new(())
            }
        }
    }
}

struct MathHandler;

impl algae::Handler<MathOp> for MathHandler {
    fn handle(&mut self, op: &MathOp) -> Box<dyn std::any::Any + Send> {
        match op {
            MathOp::MathV2(MathV2::Add((a, b))) => {
                let result = a + b;
                println!("[CUSTOM MATH] {} + {} = {}", a, b, result);
                Box::new(result)
            }
            MathOp::MathV2(MathV2::Multiply((a, b))) => {
                let result = a * b;
                println!("[CUSTOM MATH] {} * {} = {}", a, b, result);
                Box::new(result)
            }
            MathOp::MathV2(MathV2::Divide((a, b))) => {
                if *b == 0 {
                    Box::new(Err::<i32, String>("Division by zero".to_string()))
                } else {
                    let result = a / b;
                    println!("[CUSTOM MATH] {} / {} = {}", a, b, result);
                    Box::new(Ok::<i32, String>(result))
                }
            }
            MathOp::MathV2(MathV2::Power((base, exp))) => {
                let result = base.pow(*exp);
                println!("[CUSTOM MATH] {}^{} = {}", base, exp, result);
                Box::new(result)
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
        files.insert("existing.txt".to_string(), "existing content".to_string());
        Self { files }
    }
}

impl algae::Handler<FileOp> for FileHandler {
    fn handle(&mut self, op: &FileOp) -> Box<dyn std::any::Any + Send> {
        match op {
            FileOp::FileV2(FileV2::Read(path)) => {
                if let Some(content) = self.files.get(path) {
                    println!("[CUSTOM FILE] Reading {}: {}", path, content);
                    Box::new(Ok::<String, String>(content.clone()))
                } else {
                    println!("[CUSTOM FILE] File not found: {}", path);
                    Box::new(Err::<String, String>(format!("File not found: {}", path)))
                }
            }
            FileOp::FileV2(FileV2::Write((path, content))) => {
                self.files.insert(path.clone(), content.clone());
                println!("[CUSTOM FILE] Writing to {}: {}", path, content);
                Box::new(Ok::<(), String>(()))
            }
            FileOp::FileV2(FileV2::Delete(path)) => {
                if self.files.remove(path).is_some() {
                    println!("[CUSTOM FILE] Deleted: {}", path);
                    Box::new(Ok::<(), String>(()))
                } else {
                    Box::new(Err::<(), String>(format!("File not found: {}", path)))
                }
            }
        }
    }
}

struct LoggerHandler {
    count: usize,
}

impl LoggerHandler {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl algae::Handler<LoggerOp> for LoggerHandler {
    fn handle(&mut self, op: &LoggerOp) -> Box<dyn std::any::Any + Send> {
        match op {
            LoggerOp::LoggerV2(LoggerV2::Info(msg)) => {
                println!("[CUSTOM LOG INFO] {}", msg);
                self.count += 1;
                Box::new(())
            }
            LoggerOp::LoggerV2(LoggerV2::Error(msg)) => {
                println!("[CUSTOM LOG ERROR] {}", msg);
                self.count += 1;
                Box::new(())
            }
            LoggerOp::LoggerV2(LoggerV2::Debug(msg)) => {
                println!("[CUSTOM LOG DEBUG] {}", msg);
                self.count += 1;
                Box::new(())
            }
            LoggerOp::LoggerV2(LoggerV2::GetLogCount) => Box::new(self.count),
        }
    }
}

// Combine all custom root enums into one unified enum (for advanced use cases)
algae::combine_roots!(pub UnifiedOp = ConsoleOp, MathOp, FileOp, LoggerOp);

// ============================================================================
// ✅ APPROACH 3: Module-based separation (for large codebases)
// ============================================================================

mod console_module {
    use algae::prelude::*;

    // Each module has its own effect! declaration
    effect! {
        ConsoleOp::Print (String) -> ();
        ConsoleOp::ReadLine -> String;
        ConsoleOp::Clear -> ();
    }

    #[effectful]
    pub fn interactive_session() -> Vec<String> {
        let mut responses = Vec::new();

        let _: () = perform!(ConsoleOp::Clear);
        let _: () = perform!(ConsoleOp::Print("=== Interactive Session ===".to_string()));

        for i in 1..=3 {
            let _: () = perform!(ConsoleOp::Print(format!(
                "Question {}: What's your favorite color?",
                i
            )));
            let answer: String = perform!(ConsoleOp::ReadLine);
            responses.push(answer);
        }

        responses
    }

    // Module-specific handler
    pub struct ConsoleHandler {
        responses: Vec<String>,
        index: std::cell::RefCell<usize>,
    }

    impl ConsoleHandler {
        pub fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                index: std::cell::RefCell::new(0),
            }
        }
    }

    impl Handler<Op> for ConsoleHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            match op {
                Op::ConsoleOp(ConsoleOp::Print(msg)) => {
                    println!("[CONSOLE] {}", msg);
                    Box::new(())
                }
                Op::ConsoleOp(ConsoleOp::ReadLine) => {
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
                Op::ConsoleOp(ConsoleOp::Clear) => {
                    println!("[CONSOLE] <screen cleared>");
                    Box::new(())
                }
            }
        }
    }
}

mod math_module {
    use algae::prelude::*;

    effect! {
        MathOp::Add ((i32, i32)) -> i32;
        MathOp::Multiply ((i32, i32)) -> i32;
        MathOp::Divide ((i32, i32)) -> Result<i32, String>;
        MathOp::Power ((i32, u32)) -> i32;
    }

    #[effectful]
    pub fn complex_calculation(base: i32) -> Result<i32, String> {
        let doubled: i32 = perform!(MathOp::Multiply((base, 2)));
        let squared: i32 = perform!(MathOp::Power((doubled, 2)));
        let added: i32 = perform!(MathOp::Add((squared, 10)));
        let result: Result<i32, String> = perform!(MathOp::Divide((added, 3)));
        result
    }

    pub struct MathHandler;

    impl Handler<Op> for MathHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            match op {
                Op::MathOp(MathOp::Add((a, b))) => {
                    let result = a + b;
                    println!("[MATH] {} + {} = {}", a, b, result);
                    Box::new(result)
                }
                Op::MathOp(MathOp::Multiply((a, b))) => {
                    let result = a * b;
                    println!("[MATH] {} * {} = {}", a, b, result);
                    Box::new(result)
                }
                Op::MathOp(MathOp::Divide((a, b))) => {
                    if *b == 0 {
                        Box::new(Err::<i32, String>("Division by zero".to_string()))
                    } else {
                        let result = a / b;
                        println!("[MATH] {} / {} = {}", a, b, result);
                        Box::new(Ok::<i32, String>(result))
                    }
                }
                Op::MathOp(MathOp::Power((base, exp))) => {
                    let result = base.pow(*exp);
                    println!("[MATH] {}^{} = {}", base, exp, result);
                    Box::new(result)
                }
            }
        }
    }
}

// ============================================================================
// UNIFIED HANDLER for the main effect! declaration
// ============================================================================

struct UnifiedHandler {
    console_responses: Vec<String>,
    console_index: usize,
    file_contents: std::collections::HashMap<String, String>,
}

impl UnifiedHandler {
    fn new() -> Self {
        Self {
            console_responses: vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
            ],
            console_index: 0,
            file_contents: std::collections::HashMap::new(),
        }
    }
}

impl Handler<Op> for UnifiedHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Console operations
            Op::Console(Console::Print(msg)) => {
                println!("[UNIFIED CONSOLE] {}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let response = if self.console_index < self.console_responses.len() {
                    self.console_responses[self.console_index].clone()
                } else {
                    "default".to_string()
                };
                self.console_index += 1;
                println!("[UNIFIED CONSOLE] <input: {}>", response);
                Box::new(response)
            }

            // Math operations
            Op::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[UNIFIED MATH] {} + {} = {}", a, b, result);
                Box::new(result)
            }
            Op::Math(Math::Multiply((a, b))) => {
                let result = a * b;
                println!("[UNIFIED MATH] {} * {} = {}", a, b, result);
                Box::new(result)
            }
            Op::Math(Math::Divide((a, b))) => {
                if *b == 0 {
                    Box::new(Err::<i32, String>("Division by zero".to_string()))
                } else {
                    let result = a / b;
                    println!("[UNIFIED MATH] {} / {} = {}", a, b, result);
                    Box::new(Ok::<i32, String>(result))
                }
            }

            // File operations
            Op::File(File::Read(path)) => {
                let content = self
                    .file_contents
                    .get(path)
                    .cloned()
                    .unwrap_or_else(|| format!("No content found for {}", path));
                println!("[UNIFIED FILE] Reading {}: {}", path, content);
                Box::new(Ok::<String, String>(content))
            }
            Op::File(File::Write((path, content))) => {
                self.file_contents.insert(path.clone(), content.clone());
                println!("[UNIFIED FILE] Writing to {}: {}", path, content);
                Box::new(Ok::<(), String>(()))
            }

            // Logger operations
            Op::Logger(Logger::Info(msg)) => {
                println!("[UNIFIED LOG INFO] {}", msg);
                Box::new(())
            }
            Op::Logger(Logger::Error(msg)) => {
                println!("[UNIFIED LOG ERROR] {}", msg);
                Box::new(())
            }
            Op::Logger(Logger::Debug(msg)) => {
                println!("[UNIFIED LOG DEBUG] {}", msg);
                Box::new(())
            }

            // HTTP operations
            Op::Http(Http::Get(url)) => {
                println!("[UNIFIED HTTP] GET {}", url);
                Box::new(Ok::<String, String>(format!(
                    "{{\"data\": \"mock response from {}\"}}",
                    url
                )))
            }
            Op::Http(Http::Post((url, body))) => {
                println!("[UNIFIED HTTP] POST {} with body: {}", url, body);
                Box::new(Ok::<String, String>(
                    "{\"status\": \"success\"}".to_string(),
                ))
            }
        }
    }
}

// ============================================================================
// DEMONSTRATION
// ============================================================================

fn main() {
    println!("=== Multiple Effects Patterns Demo ===\n");

    println!("1. Single effect! macro with multiple families (traditional approach):");
    let result1 = comprehensive_demo().handle(UnifiedHandler::new()).run();
    println!("Traditional result: {}\n", result1);

    println!("2. Multiple effect! with custom root names:");

    // Console demo
    let console_result = console_demo_v2()
        .handle(ConsoleHandler::new(vec!["Alice".to_string()]))
        .run();
    println!("Custom Console result: {}", console_result);

    // Math demo
    let math_result = math_demo_v2(7, 3).handle(MathHandler).run();
    println!("Custom Math result: {}", math_result);

    // File demo
    let file_result = file_demo_v2().handle(FileHandler::new()).run();
    println!("Custom File result: {}", file_result);

    // Logger demo
    let logger_result = logger_demo_v2().handle(LoggerHandler::new()).run();
    println!("Custom Logger result: {} log entries\n", logger_result);

    println!("3. Module-based separation (for large codebases):");

    // Console module demo
    let console_responses = console_module::interactive_session()
        .handle(console_module::ConsoleHandler::new(vec![
            "Blue".to_string(),
            "Green".to_string(),
            "Red".to_string(),
        ]))
        .run();
    println!("Console responses: {:?}", console_responses);

    // Math module demo
    let math_result = math_module::complex_calculation(5)
        .handle(math_module::MathHandler)
        .run();
    println!("Math result: {:?}\n", math_result);

    println!("=== Approaches Summary ===");
    println!("✅ APPROACH 1: Single effect! with multiple families");
    println!("   - Traditional approach, simplest for small-medium projects");
    println!("   - All effects in one unified enum");
    println!("   - Single handler implementation");
    println!("");
    println!("✅ APPROACH 2: Multiple effect! with custom root names");
    println!("   - Best for modular organization in same module");
    println!("   - Each effect family has its own root enum");
    println!("   - Use combine_roots! for unified handling");
    println!("   - Clear separation of concerns");
    println!("");
    println!("✅ APPROACH 3: Module-based separation");
    println!("   - Best for large teams and codebases");
    println!("   - Each module owns its effects");
    println!("   - Natural namespace separation");
    println!("   - Independent testing and development");
    println!("");
    println!("🎯 Choose based on your needs:");
    println!("   • Small project: Approach 1");
    println!("   • Organized same-module effects: Approach 2");
    println!("   • Large codebase/team: Approach 3");
}
