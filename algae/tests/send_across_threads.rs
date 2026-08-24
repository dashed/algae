#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: `Effectful` values cross thread boundaries.
//!
//! `EffectCoroutine` carries a `+ Send` bound, so a computation can be built on
//! one thread and handled on another. Without that bound the `thread::spawn`
//! calls below fail to compile.

use algae::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

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
    logged: Arc<Mutex<Vec<String>>>,
}

impl Handler<Op> for ThreadSafeHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Compute(Compute::Add((a, b))) => Box::new(a + b),
            Op::Compute(Compute::Multiply((a, b))) => Box::new(a * b),
            Op::Logger(Logger::Log(msg)) => {
                self.logged.lock().unwrap().push(msg.clone());
                Box::new(())
            }
        }
    }
}

#[test]
fn a_computation_built_here_can_be_handled_on_another_thread() {
    let logged = Arc::new(Mutex::new(Vec::new()));
    let computation = compute_in_thread(10, 20);

    let handler_log = Arc::clone(&logged);
    let handle = thread::spawn(move || {
        computation
            .handle(ThreadSafeHandler {
                logged: handler_log,
            })
            .run()
    });

    let result = handle.join().expect("worker thread panicked");

    assert_eq!(result, 60);
    assert_eq!(
        logged.lock().unwrap().as_slice(),
        ["Computing 10 + 20", "Computing 30 * 2", "Final result: 60"]
    );
}

#[test]
fn many_computations_run_concurrently() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let computation = compute_in_thread(i * 10, i * 5);
            thread::spawn(move || {
                computation
                    .handle(ThreadSafeHandler {
                        logged: Arc::new(Mutex::new(Vec::new())),
                    })
                    .run()
            })
        })
        .collect();

    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().expect("worker thread panicked"))
        .collect();

    // (i * 10 + i * 5) * 2 == i * 30
    assert_eq!(results, vec![0, 30, 60]);
}

/// Fails to compile if the `Send` bound is ever dropped from the coroutine.
#[test]
fn effectful_is_send() {
    fn assert_send<T: Send>(_: T) {}

    assert_send(compute_in_thread(1, 2));
}
