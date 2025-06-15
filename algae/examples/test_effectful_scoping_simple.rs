//! Test simple scoping issue with #[effectful] in nested modules
//!
//! This example tests the specific case where #[effectful] doesn't work
//! when the effect is defined in one scope but the function is in another.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define effects at the top level
effect! {
    Console::Print (String) -> ();
    Math::Add ((i32, i32)) -> i32;
}

mod inner {
    use super::{Console, Math, Op};
    use algae::prelude::*; // Import the types from parent module

    // This should work now that we import Op
    #[effectful]
    pub fn inner_function() -> String {
        let _: () = perform!(Console::Print("Hello from inner module".to_string()));
        let result: i32 = perform!(Math::Add((10, 20)));
        format!("Result: {result}")
    }
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[HANDLER] {msg}");
                Box::new(())
            }
            Op::Math(Math::Add((a, b))) => {
                let result = a + b;
                println!("[HANDLER] {a} + {b} = {result}");
                Box::new(result)
            }
        }
    }
}

fn main() {
    println!("Testing simple scoping with #[effectful]...");

    let result = inner::inner_function().handle(TestHandler).run();

    println!("Final result: {result}");
    println!("✅ Success!");
}
