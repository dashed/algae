#![feature(yield_expr, coroutines)]
use algae::prelude::*;
use std::io::{self, Write};

effect! {
    Console::Print    (String)                 -> ();
    Console::ReadLine                          -> String;

    Random::Int (std::ops::Range<i32>)         -> i32;
}

// ---------- handlers -------------------------------------------------
pub struct RealConsole;
impl Handler<Op> for RealConsole {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("{msg}");
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                print!("Enter your name: ");
                io::stdout().flush().unwrap();
                let mut buf = String::new();
                io::stdin().read_line(&mut buf).unwrap();
                Box::new(buf.trim().to_string())
            }
            _ => panic!("RealConsole cannot handle non-Console operations"),
        }
    }
}

pub struct MockConsole {
    responses: Vec<String>,
    index: usize,
}

impl MockConsole {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            index: 0,
        }
    }
}

impl Handler<Op> for MockConsole {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("[MOCK] {msg}");
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let response = if self.index < self.responses.len() {
                    self.responses[self.index].clone()
                } else {
                    "default".to_string()
                };
                self.index += 1;
                Box::new(response)
            }
            _ => panic!("MockConsole cannot handle non-Console operations"),
        }
    }
}

pub struct Rand {
    rng: u64, // Simple LCG for demonstration
}

impl Rand {
    pub fn seeded(seed: u64) -> Self {
        Self { rng: seed }
    }

    fn next(&mut self) -> u64 {
        self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
        self.rng
    }
}

impl Handler<Op> for Rand {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Random(Random::Int(range)) => {
                let val = (self.next() % (range.end - range.start) as u64) as i32 + range.start;
                Box::new(val)
            }
            _ => panic!("Rand cannot handle non-Random operations"),
        }
    }
}

// ---------- business logic ------------------------------------------
#[effectful]
fn program() -> i32 {
    let name: String = perform!(Console::ReadLine);
    let _: () = perform!(Console::Print(format!("Hello, {}!", name)));
    let lucky: i32 = perform!(Random::Int(1..10));
    let _: () = perform!(Console::Print(format!("Your lucky number is {}", lucky)));
    lucky
}

// Combined handler that handles both Console and Random effects
pub struct CombinedHandler {
    console_handler: RealConsole,
    rand_handler: Rand,
}

impl Default for CombinedHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CombinedHandler {
    pub fn new() -> Self {
        Self {
            console_handler: RealConsole,
            rand_handler: Rand::seeded(42),
        }
    }
}

impl Handler<Op> for CombinedHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(_) => self.console_handler.handle(op),
            Op::Random(_) => self.rand_handler.handle(op),
        }
    }
}

pub struct MockCombinedHandler {
    console_handler: MockConsole,
    rand_handler: Rand,
}

impl MockCombinedHandler {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            console_handler: MockConsole::new(responses),
            rand_handler: Rand::seeded(42),
        }
    }
}

impl Handler<Op> for MockCombinedHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(_) => self.console_handler.handle(op),
            Op::Random(_) => self.rand_handler.handle(op),
        }
    }
}

fn main() {
    println!("=== Interactive Console Demo ===");
    let answer = program().handle(CombinedHandler::new()).run();
    println!("Final result: {answer}");

    println!("\n=== Mock Console Demo ===");
    let mock_answer = program()
        .handle(MockCombinedHandler::new(vec!["Alice".to_string()]))
        .run();
    println!("Mock result: {mock_answer}");
}
