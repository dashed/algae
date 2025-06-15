//! Demonstrating multiple effect! macro usage patterns
//! 
//! This example shows the current limitations and best practices
//! for organizing effects in larger applications.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// ============================================================================
// ❌ PROBLEM: This would NOT work (compilation error)
// ============================================================================

/*
// Multiple effect! macros in same scope create conflicting Op enum definitions:

effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

effect! {
    Math::Add ((i32, i32)) -> i32;
    Math::Multiply ((i32, i32)) -> i32;
}

// ERROR: Both macros generate their own `pub enum Op { ... }`
// Result: "duplicate definition of `Op`" compile error
*/

// ============================================================================
// ✅ SOLUTION 1: Single effect! macro with multiple families (RECOMMENDED)
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
// ✅ SOLUTION 2: Module-based separation (for organization)
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
            let _: () = perform!(ConsoleOp::Print(format!("Question {}: What's your favorite color?", i)));
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
                    let response = self.responses.get(*index).cloned()
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
                "Charlie".to_string()
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
                let content = self.file_contents.get(path).cloned()
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
                Box::new(Ok::<String, String>(format!("{{\"data\": \"mock response from {}\"}}", url)))
            }
            Op::Http(Http::Post((url, body))) => {
                println!("[UNIFIED HTTP] POST {} with body: {}", url, body);
                Box::new(Ok::<String, String>("{\"status\": \"success\"}".to_string()))
            }
        }
    }
}

// ============================================================================
// DEMONSTRATION
// ============================================================================

fn main() {
    println!("=== Multiple Effects Patterns Demo ===\n");
    
    println!("1. Single effect! macro with multiple families (RECOMMENDED):");
    let result1 = comprehensive_demo()
        .handle(UnifiedHandler::new())
        .run();
    println!("Unified result: {}\n", result1);
    
    println!("2. Module-based separation (for large codebases):");
    
    // Console module demo
    let console_responses = console_module::interactive_session()
        .handle(console_module::ConsoleHandler::new(vec![
            "Blue".to_string(),
            "Green".to_string(), 
            "Red".to_string()
        ]))
        .run();
    println!("Console responses: {:?}", console_responses);
    
    // Math module demo
    let math_result = math_module::complex_calculation(5)
        .handle(math_module::MathHandler)
        .run();
    println!("Math result: {:?}\n", math_result);
    
    println!("=== Best Practices Summary ===");
    println!("✅ RECOMMENDED: Single effect! with multiple families");
    println!("   - All effects in one enum");
    println!("   - Single unified handler");
    println!("   - Easy composition and testing");
    println!("");
    println!("✅ ALTERNATIVE: Module separation");
    println!("   - Separate effect! per module");
    println!("   - Good for large teams/codebases");
    println!("   - Each module handles its own effects");
    println!("");
    println!("❌ NOT SUPPORTED: Multiple effect! in same scope");
    println!("   - Creates conflicting Op enum definitions");
    println!("   - Compilation error");
    println!("");
    println!("📁 Use modules for:");
    println!("   - Different feature areas");
    println!("   - Team ownership boundaries");
    println!("   - Independent testing");
}