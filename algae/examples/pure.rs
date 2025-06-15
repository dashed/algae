#![feature(yield_expr, coroutines)]
use algae::prelude::*;

effect! {
    State::Get -> i32;
    State::Put (i32) -> ();
}

pub struct StateHandler {
    value: i32,
}

impl StateHandler {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

impl Handler<Op> for StateHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::State(State::Get) => Box::new(self.value),
            Op::State(State::Put(new_val)) => {
                self.value = *new_val;
                Box::new(())
            }
        }
    }
}

#[effectful]
fn counter_program() -> i32 {
    let current: i32 = perform!(State::Get);
    let _: () = perform!(State::Put(current + 1));
    let incremented: i32 = perform!(State::Get);
    let _: () = perform!(State::Put(incremented * 2));
    perform!(State::Get)
}

fn main() {
    println!("=== Pure State Example ===");
    let result = counter_program()
        .handle(StateHandler::new(5))
        .run();
    println!("Result: {result}"); // Should be (5 + 1) * 2 = 12
    
    // Test with different initial state
    let result2 = counter_program()
        .handle(StateHandler::new(10))
        .run();
    println!("Result with initial 10: {result2}"); // Should be (10 + 1) * 2 = 22
}