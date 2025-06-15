#![feature(coroutines, coroutine_trait, yield_expr)]

use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

fn main() {
    let mut gen = Box::pin(
        #[coroutine]
        |_: i32| {
            let result: i32 = yield "hello";
            result + 1
        },
    );

    // Start the coroutine with a dummy value
    match Pin::as_mut(&mut gen).resume(0) {
        CoroutineState::Yielded(msg) => {
            println!("Got message: {msg}");
            // Resume with a value
            match Pin::as_mut(&mut gen).resume(42) {
                CoroutineState::Complete(result) => println!("Final result: {result}"),
                _ => panic!("Expected completion"),
            }
        }
        _ => panic!("Expected yielded value"),
    }
}
