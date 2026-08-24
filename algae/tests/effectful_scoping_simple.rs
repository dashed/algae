#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: `#[effectful]` used from a nested module.
//!
//! The effects are declared at the crate root while the `#[effectful]` function
//! lives in an inner module. The expansion names the generated root type, so the
//! inner module has to bring it into scope; this test pins that arrangement down.

use algae::prelude::*;
use std::sync::{Arc, Mutex};

effect! {
    Console::Print (String) -> ();
    Math::Add ((i32, i32)) -> i32;
}

mod inner {
    use super::{Console, Math, Op};
    use algae::prelude::*;

    #[effectful]
    pub fn inner_function() -> String {
        let _: () = perform!(Console::Print("Hello from inner module".to_string()));
        let result: i32 = perform!(Math::Add((10, 20)));
        format!("Result: {result}")
    }
}

struct TestHandler {
    printed: Arc<Mutex<Vec<String>>>,
}

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                self.printed.lock().unwrap().push(msg.clone());
                Box::new(())
            }
            Op::Math(Math::Add((a, b))) => Box::new(a + b),
        }
    }
}

#[test]
fn effectful_resolves_when_defined_in_a_nested_module() {
    let printed = Arc::new(Mutex::new(Vec::new()));

    let result = inner::inner_function()
        .handle(TestHandler {
            printed: Arc::clone(&printed),
        })
        .run();

    assert_eq!(result, "Result: 30");
    assert_eq!(
        printed.lock().unwrap().as_slice(),
        ["Hello from inner module"]
    );
}
