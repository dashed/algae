//! Example of using algae without the macros feature
//!
//! This demonstrates the manual approach to defining effects and effectful
//! computations without using the `effect!`, `#[effectful]`, and `perform!` macros.
//!
//! To run this example with macros disabled:
//! ```
//! cargo run --example no_macros --no-default-features
//! ```

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;
use std::any::Any;

// Manually define effect operations (equivalent to what effect! macro generates)
#[derive(Debug)]
pub enum Console {
    Print(String),
    ReadLine,
}

impl Default for Console {
    fn default() -> Self {
        Console::ReadLine
    }
}

#[derive(Debug)]
pub enum Op {
    Console(Console),
}

impl Default for Op {
    fn default() -> Self {
        Op::Console(Console::default())
    }
}

impl From<Console> for Op {
    fn from(c: Console) -> Self {
        Op::Console(c)
    }
}

// Handler implementation
struct MockConsole {
    responses: Vec<String>,
    index: std::cell::RefCell<usize>,
}

impl MockConsole {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: std::cell::RefCell::new(0),
        }
    }
}

impl Handler<Op> for MockConsole {
    fn handle(&mut self, op: &Op) -> Box<dyn Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[MOCK] {}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let mut index = self.index.borrow_mut();
                let response = self
                    .responses
                    .get(*index)
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                *index += 1;
                Box::new(response)
            }
        }
    }
}

// Manually create effectful computation (equivalent to what #[effectful] macro generates)
fn greet_user() -> Effectful<String, Op> {
    Effectful::new(
        #[coroutine]
        |mut _reply: Option<Reply>| {
            // Print prompt (equivalent to perform!(Console::Print(...)))
            {
                let __eff = Effect::new(Console::Print("What's your name?".to_string()).into());
                let __reply_opt = yield __eff;
                let _: () = __reply_opt.unwrap().take::<()>();
            }

            // Read input (equivalent to perform!(Console::ReadLine))
            let name: String = {
                let __eff = Effect::new(Console::ReadLine.into());
                let __reply_opt = yield __eff;
                __reply_opt.unwrap().take::<String>()
            };

            format!("Hello, {}!", name)
        },
    )
}

fn main() {
    println!("=== No Macros Example ===");
    println!("This example demonstrates using algae without the macros feature.");

    let handler = MockConsole::new(vec!["Alice".to_string()]);
    let result = greet_user().handle(handler).run();

    println!("Result: {}", result);
    println!("\nAs you can see, the core functionality works without macros,");
    println!("but the syntax is much more verbose!");
}
