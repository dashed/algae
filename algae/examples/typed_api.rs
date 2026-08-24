//! The typed API: `effect!`-generated constructors and handler traits.
//!
//! `effect!` generates, alongside the enums:
//! - a snake_case constructor per operation (`Console::read_line()`) that
//!   carries the declared reply type, so `perform!` needs no annotations and
//!   a wrong-type request is a compile error;
//! - a typed handler trait per family (`ConsoleOps`) plus an adapter
//!   (`HandleConsole`) that turns any implementation into a `PartialHandler`
//!   — no boxing, no `match`, reply types checked at compile time.
//!
//! The legacy forms (`perform!(Console::Print(msg))` with an annotation, and
//! hand-written `Handler` impls) keep working unchanged.
#![feature(coroutines, yield_expr)]
use algae::prelude::*;

effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
    Math::Add ((i32, i32)) -> i32;
}

#[effectful]
fn program() -> String {
    // No `let _: () = ...`, no type annotations: the constructors carry the
    // declared reply types.
    perform!(Console::print("What's your name?".to_string()));
    let name = perform!(Console::read_line());
    let lucky = perform!(Math::add(3, 4)); // tuple payload -> named parameters
    format!("Hello, {name}! Your number is {lucky}.")
}

// Typed handlers: implement the generated trait, one typed method per op.
struct MockConsole {
    input: String,
}
impl ConsoleOps for MockConsole {
    fn print(&mut self, value: &String) {
        println!("[mock] {value}");
    }
    fn read_line(&mut self) -> String {
        self.input.clone()
    }
}

struct RealMath;
impl MathOps for RealMath {
    fn add(&mut self, v0: &i32, v1: &i32) -> i32 {
        v0 + v1
    }
}

fn main() {
    let result = program()
        .begin_chain()
        .handle(HandleConsole(MockConsole {
            input: "Ada".to_string(),
        }))
        .handle(HandleMath(RealMath))
        .run_checked()
        .expect("all families handled");

    assert_eq!(result, "Hello, Ada! Your number is 7.");
    println!("{result}");
}
