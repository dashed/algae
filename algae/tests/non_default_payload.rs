#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

//! Regression test: payload types that do not implement `Default`.
//!
//! `effect!` used to generate `Default` impls for every effect family without
//! bounding the payloads, so a payload lacking `Default` broke compilation. The
//! payload below has no `Default`, which makes this file a compile-time check on
//! its own; the assertions additionally confirm the effects still round-trip.

use algae::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct NonDefaultType {
    pub value: String,
    pub id: u64,
}

effect! {
    FileOps::ReadFile (NonDefaultType) -> String;
    FileOps::WriteFile ((NonDefaultType, String)) -> Result<(), String>;
    NetworkOps::HttpGet (NonDefaultType) -> Result<String, String>;
}

#[effectful]
fn test_function() -> String {
    let non_default = NonDefaultType {
        value: "test".to_string(),
        id: 42,
    };

    let content: String = perform!(FileOps::ReadFile(non_default.clone()));
    let written: Result<(), String> = perform!(FileOps::WriteFile((
        non_default.clone(),
        "content".to_string()
    )));
    let response: Result<String, String> = perform!(NetworkOps::HttpGet(non_default));

    assert_eq!(content, "file content");
    assert_eq!(written, Ok(()));
    assert_eq!(response, Ok("response".to_string()));

    "Success".to_string()
}

struct TestHandler;

impl Handler<Op> for TestHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileOps(FileOps::ReadFile(file_info)) => {
                assert_eq!(file_info.id, 42);
                Box::new("file content".to_string())
            }
            Op::FileOps(FileOps::WriteFile((file_info, content))) => {
                assert_eq!(file_info.value, "test");
                assert_eq!(content, "content");
                Box::new(Ok::<(), String>(()))
            }
            Op::NetworkOps(NetworkOps::HttpGet(request_info)) => {
                assert_eq!(request_info.value, "test");
                Box::new(Ok::<String, String>("response".to_string()))
            }
        }
    }
}

#[test]
fn effects_compile_and_run_with_payloads_that_lack_default() {
    let result = test_function().handle(TestHandler).run();

    assert_eq!(result, "Success");
}
