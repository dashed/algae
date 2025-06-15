#![feature(coroutines, coroutine_trait, yield_expr)]

use algae::prelude::*;

effect! {
    TestEffect::GetString -> String;
}

struct StringHandler;

impl Handler<Op> for StringHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        println!("Handler called with op: {op:?}");
        match op {
            Op::TestEffect(TestEffect::GetString) => {
                let result = "test result".to_string();
                println!("Handler returning String: {result:?}");
                Box::new(result)
            }
        }
    }
}

fn main() {
    // Test the handler directly first
    let mut handler = StringHandler;
    let test_op = Op::TestEffect(TestEffect::GetString);
    let _boxed_result = handler.handle(&test_op);
    println!("Direct handler test successful");

    // Test Effect creation and take
    let mut effect = Effect::new(test_op);
    effect.fill_boxed(Box::new("direct test".to_string()));
    let reply = effect.get_reply();
    let result: String = reply.take();
    println!("Direct effect test: {result}");

    // Now test the full effectful function
    #[effectful]
    fn test_effectful() -> String {
        perform!(TestEffect::GetString)
    }

    let result = test_effectful().handle(StringHandler).run();
    println!("Effectful test result: {result}");
}
