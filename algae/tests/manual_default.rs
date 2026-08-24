#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: hand-written `Default` impls for generated effect types.
//!
//! `effect!` deliberately stops generating `Default` (see
//! `non_default_payload.rs` for why). Nothing stops a user from writing those
//! impls themselves, so the generated types must leave the slot free rather
//! than occupy it.

use algae::prelude::*;

effect! {
    SimpleOps::NoPayload -> String;
    SimpleOps::WithPayload (i32) -> String;
}

#[allow(clippy::derivable_impls)] // Intentional manual implementation for demonstration
impl Default for SimpleOps {
    fn default() -> Self {
        SimpleOps::NoPayload
    }
}

impl Default for Op {
    fn default() -> Self {
        Op::SimpleOps(SimpleOps::default())
    }
}

#[effectful]
fn test_function() -> String {
    let result1: String = perform!(SimpleOps::NoPayload);
    let result2: String = perform!(SimpleOps::WithPayload(42));
    format!("{result1} / {result2}")
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::SimpleOps(SimpleOps::NoPayload) => Box::new("No payload result".to_string()),
            Op::SimpleOps(SimpleOps::WithPayload(value)) => {
                Box::new(format!("Payload result: {value}"))
            }
        }
    }
}

#[test]
fn hand_written_default_impls_are_accepted() {
    assert!(matches!(SimpleOps::default(), SimpleOps::NoPayload));
    assert!(matches!(Op::default(), Op::SimpleOps(SimpleOps::NoPayload)));
}

#[test]
fn effects_still_work_with_a_manual_default_in_scope() {
    let result = test_function().handle(TestHandler).run();

    assert_eq!(result, "No payload result / Payload result: 42");
}
