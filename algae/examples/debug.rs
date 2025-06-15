#![feature(yield_expr, coroutines)]
use algae::prelude::*;

// First, let's test if the effect macro works
effect! {
    Test::Hello -> String;
}

#[effectful]
fn test_function() -> String {
    let result: String = perform!(Test::Hello);
    result
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Test(Test::Hello) => {
                let result = "Hello from handler!".to_string();
                println!(
                    "Handler returning String with TypeId: {:?}",
                    (&result as &dyn std::any::Any).type_id()
                );
                Box::new(result)
            }
        }
    }
}

fn main() {
    println!("Effect macro compiled successfully!");

    // Debug type IDs
    use std::any::{Any, TypeId};
    let string_type = TypeId::of::<String>();
    println!("String TypeId: {string_type:?}");
    let test_string = "Hello from handler!".to_string();
    println!(
        "test_string TypeId: {:?}",
        (&test_string as &dyn Any).type_id()
    );

    // Test other types that might match the unknown TypeId
    let unit_type = TypeId::of::<()>();
    println!("() TypeId: {unit_type:?}");
    let i32_type = TypeId::of::<i32>();
    println!("i32 TypeId: {i32_type:?}");
    let test_type = TypeId::of::<Test>();
    println!("Test::Hello TypeId: {test_type:?}");
    let op_type = TypeId::of::<Op>();
    println!("Op TypeId: {op_type:?}");

    // Print the generated types
    let op = Test::Hello;
    println!("Created op: {op:?}");

    let main_op: Op = op.into();
    println!("Converted to main op: {main_op:?}");

    // Test effectful function with handler
    let result = test_function().handle(TestHandler).run();
    println!("Result from effectful function: {result}");
}
