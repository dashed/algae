//! Test that Effectful computations can be sent across threads
//!
//! This example demonstrates that with the + Send bound added to EffectCoroutine,
//! effectful computations can now be safely transferred between threads.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;
use std::thread;

// Define effects for testing
effect! {
    Compute::Add ((i32, i32)) -> i32;
    Compute::Multiply ((i32, i32)) -> i32;
    Logger::Log (String) -> ();
}

#[effectful]
fn compute_in_thread(x: i32, y: i32) -> i32 {
    let _: () = perform!(Logger::Log(format!("Computing {x} + {y}")));
    let sum: i32 = perform!(Compute::Add((x, y)));

    let _: () = perform!(Logger::Log(format!("Computing {sum} * 2")));
    let result: i32 = perform!(Compute::Multiply((sum, 2)));

    let _: () = perform!(Logger::Log(format!("Final result: {result}")));
    result
}

struct ThreadSafeHandler {
    thread_id: String,
}

impl ThreadSafeHandler {
    fn new(thread_name: &str) -> Self {
        Self {
            thread_id: thread_name.to_string(),
        }
    }
}

impl Handler<Op> for ThreadSafeHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Compute(Compute::Add((a, b))) => {
                let result = a + b;
                println!("[{}] Computing: {} + {} = {}", self.thread_id, a, b, result);
                Box::new(result)
            }
            Op::Compute(Compute::Multiply((a, b))) => {
                let result = a * b;
                println!("[{}] Computing: {} * {} = {}", self.thread_id, a, b, result);
                Box::new(result)
            }
            Op::Logger(Logger::Log(msg)) => {
                println!("[{}] LOG: {}", self.thread_id, msg);
                Box::new(())
            }
        }
    }
}

fn main() {
    println!("Testing Effectful computations across threads...");

    // Create an effectful computation
    let computation = compute_in_thread(10, 20);

    // Send it to another thread - this should compile now with + Send
    let handle = thread::spawn(move || {
        println!("[THREAD] Starting computation in spawned thread");
        let handler = ThreadSafeHandler::new("WORKER");
        computation.handle(handler).run()
    });

    // Wait for the thread to complete
    let result = handle.join().expect("Thread should complete successfully");
    println!("[MAIN] Result from worker thread: {result}");

    // Test multiple threads
    let mut handles = vec![];
    for i in 0..3 {
        let computation = compute_in_thread(i * 10, i * 5);
        let handle = thread::spawn(move || {
            let handler = ThreadSafeHandler::new(&format!("WORKER-{i}"));
            computation.handle(handler).run()
        });
        handles.push(handle);
    }

    // Collect results from all threads
    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    println!("[MAIN] Results from all worker threads: {results:?}");

    // Verify Send trait is properly implemented
    fn assert_send<T: Send>(_: T) {}
    assert_send(compute_in_thread(1, 2));

    println!("✅ Success! Effectful computations can be sent across threads");
}
