#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// 1. Define your effects
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// 2. Write effectful functions
#[effectful]
fn greet_user() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    format!("Hello, {}!", name)
}

// 3. Implement handlers
struct RealConsoleHandler;

impl Handler<Op> for RealConsoleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("{}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Box::new(input.trim().to_string())
            }
        }
    }
}

// Mock handler for testing
struct MockConsoleHandler {
    responses: Vec<String>,
    index: std::cell::RefCell<usize>,
}

impl MockConsoleHandler {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: std::cell::RefCell::new(0),
        }
    }
}

impl Handler<Op> for MockConsoleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[MOCK] {}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let mut index = self.index.borrow_mut();
                let response = self.responses.get(*index).cloned()
                    .unwrap_or_else(|| "default".to_string());
                *index += 1;
                Box::new(response)
            }
        }
    }
}

// 4. Run with different handlers
fn main() {
    println!("=== Production Example (Real I/O) ===");
    println!("Type your name when prompted:");
    
    // Production: use real I/O
    let result = greet_user()
        .handle(RealConsoleHandler)
        .run();
    
    println!("Result: {}", result);
    
    println!("\n=== Testing Example (Mock I/O) ===");
    
    // Testing: use mock I/O
    let mock_handler = MockConsoleHandler::new(vec!["Alice".to_string()]);
    let mock_result = greet_user()
        .handle(mock_handler)
        .run();
    
    println!("Mock Result: {}", mock_result);
    assert_eq!(mock_result, "Hello, Alice!");
    
    println!("\nBoth examples completed successfully!");
}