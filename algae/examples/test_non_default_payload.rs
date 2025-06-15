//! Test that effect! macro works correctly with non-Default payload types
//!
//! This example tests the bug fix where Default implementations were being
//! generated for effect families without proper bounds, causing compilation
//! errors when payload types don't implement Default.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// A type that deliberately doesn't implement Default
#[derive(Debug, Clone, PartialEq)]
pub struct NonDefaultType {
    pub value: String,
    pub id: u64,
}

// This should compile successfully now that we don't generate Default impls
effect! {
    FileOps::ReadFile (NonDefaultType) -> String;
    FileOps::WriteFile ((NonDefaultType, String)) -> Result<(), String>;
    NetworkOps::HttpGet (NonDefaultType) -> Result<String, String>;
}

#[effectful]
fn test_function() -> String {
    let non_default = NonDefaultType {
        value: "test".to_string(),
        id: 42,
    };

    let _content: String = perform!(FileOps::ReadFile(non_default.clone()));
    let _result: Result<(), String> = perform!(FileOps::WriteFile((
        non_default.clone(),
        "content".to_string()
    )));
    let _response: Result<String, String> = perform!(NetworkOps::HttpGet(non_default));

    "Success".to_string()
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileOps(FileOps::ReadFile(file_info)) => {
                println!("Reading file: {} (id: {})", file_info.value, file_info.id);
                Box::new("file content".to_string())
            }
            Op::FileOps(FileOps::WriteFile((file_info, content))) => {
                println!(
                    "Writing to file: {} (id: {}) content: {}",
                    file_info.value, file_info.id, content
                );
                Box::new(Ok::<(), String>(()))
            }
            Op::NetworkOps(NetworkOps::HttpGet(request_info)) => {
                println!("HTTP GET: {} (id: {})", request_info.value, request_info.id);
                Box::new(Ok::<String, String>("response".to_string()))
            }
        }
    }
}

fn main() {
    println!("Testing effect! macro with non-Default payload types...");

    let result = test_function().handle(TestHandler).run();
    println!("Result: {result}");

    println!("✅ Success! effect! macro works with non-Default payload types");

    // This would have failed before the fix because the generated Default
    // implementations would have required NonDefaultType: Default

    // Note: We can't call Op::default() or FileOps::default() anymore,
    // but that's the correct behavior - users should create effects explicitly
}
