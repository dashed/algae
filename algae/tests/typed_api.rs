//! Wave-2 typed API: `effect!`-generated constructors with fully inferred
//! `perform!`, and per-family typed handler traits with adapters.
#![feature(coroutines, yield_expr)]
#![cfg(feature = "macros")]

use algae::prelude::*;

effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;

    Math::Add ((i32, i32)) -> i32;
    Math::Clamp ((i32, i32, i32)) -> i32;

    IO::ReadBytes (usize) -> Vec<u8>;
}

// ============================================================================
// Typed perform: zero annotations, tuple payloads flatten into parameters
// ============================================================================

#[effectful]
fn greet() -> String {
    // Unit-returning op: no `let _: () = ...` dance.
    perform!(Console::print("What's your name?".to_string()));
    // Reply types inferred from the constructors.
    let name = perform!(Console::read_line());
    let n = perform!(Math::add(1, 2));
    let clamped = perform!(Math::clamp(n, 0, 2));
    format!("Hello, {name}! ({clamped})")
}

#[effectful]
fn mixed_forms() -> i32 {
    // Typed and legacy forms coexist in one function through one perform!.
    let a = perform!(Math::add(1, 2));
    let b: i32 = perform!(Math::Add((10, 20)));
    a + b
}

// ============================================================================
// Typed handler traits: no boxing, no match, compile-checked reply types
// ============================================================================

struct MockConsole {
    input: String,
    printed: Vec<String>,
}

impl ConsoleOps for MockConsole {
    fn print(&mut self, value: &String) {
        self.printed.push(value.clone());
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
    fn clamp(&mut self, v0: &i32, v1: &i32, v2: &i32) -> i32 {
        *v0.max(v1).min(v2)
    }
}

#[test]
fn typed_perform_with_typed_handlers() {
    let result = greet()
        .begin_chain()
        .handle(HandleConsole(MockConsole {
            input: "Alice".to_string(),
            printed: Vec::new(),
        }))
        .handle(HandleMath(RealMath))
        .run_checked();
    assert_eq!(result.unwrap(), "Hello, Alice! (2)");
}

#[test]
fn typed_and_legacy_forms_coexist() {
    let result = mixed_forms().run_checked(HandleMath(RealMath));
    assert_eq!(result, Ok(33));
}

#[test]
fn typed_adapter_declines_foreign_families() {
    // HandleMath alone cannot answer Console ops: run_checked reports it.
    let result = greet().run_checked(HandleMath(RealMath));
    match result {
        Err(UnhandledOp(Op::Console(Console::Print(_)))) => {}
        other => panic!("expected unhandled Console::Print, got {other:?}"),
    }
}

#[test]
fn ops_trait_forwards_through_mut_ref_for_inspection() {
    let mut mock = MockConsole {
        input: "Bob".to_string(),
        printed: Vec::new(),
    };

    #[effectful]
    fn console_only() -> String {
        perform!(Console::print("hi".to_string()));
        perform!(Console::read_line())
    }

    // HandleConsole(&mut mock): the generated `impl ConsoleOps for &mut T`
    // lets the mock survive the run for inspection.
    let result = console_only().run_checked(HandleConsole(&mut mock));
    assert_eq!(result, Ok("Bob".to_string()));
    assert_eq!(mock.printed, ["hi"]);
}

#[test]
fn acronym_family_gets_clean_snake_case() {
    // Family `IO` generates `IO::read_bytes`, trait `IOOps`, adapter `HandleIO`.
    #[effectful]
    fn read() -> Vec<u8> {
        perform!(IO::read_bytes(3))
    }

    struct FixedIO;
    impl IOOps for FixedIO {
        fn read_bytes(&mut self, value: &usize) -> Vec<u8> {
            vec![7u8; *value]
        }
    }

    let result = read().run_checked(HandleIO(FixedIO));
    assert_eq!(result, Ok(vec![7, 7, 7]));
}

// ============================================================================
// Custom roots and keyword-colliding variant names
// ============================================================================

mod custom_root {
    use super::*;

    effect! {
        root RobotOp;
        Robot::Move ((i32, i32)) -> bool;
        Robot::Halt -> ();
    }

    #[effectful(root = RobotOp)]
    fn drive() -> bool {
        // `Move` snake-cases to the keyword `move`; the generated
        // constructor is a raw identifier.
        let ok = perform!(Robot::r#move(3, 4));
        perform!(Robot::halt());
        ok
    }

    struct Sim {
        moves: Vec<(i32, i32)>,
        halted: bool,
    }
    impl RobotOps for Sim {
        fn r#move(&mut self, v0: &i32, v1: &i32) -> bool {
            self.moves.push((*v0, *v1));
            true
        }
        fn halt(&mut self) {
            self.halted = true;
        }
    }

    #[test]
    fn typed_api_with_custom_root_and_keyword_variant() {
        let mut sim = Sim {
            moves: Vec::new(),
            halted: false,
        };
        let result = drive().run_checked(HandleRobot(&mut sim));
        assert_eq!(result, Ok(true));
        assert_eq!(sim.moves, [(3, 4)]);
        assert!(sim.halted);
    }
}

// ============================================================================
// Typed perform still composes with call! and ScriptHandler
// ============================================================================

#[test]
fn typed_perform_through_call_and_script() {
    #[effectful]
    fn sub() -> i32 {
        perform!(Math::add(20, 2))
    }

    #[effectful]
    fn top() -> i32 {
        perform!(Console::print("computing".to_string()));
        let n: i32 = call!(sub());
        n * 2
    }

    let mut script = ScriptHandler::named("typed")
        .reply(()) // Console::print
        .reply(22i32); // Math::add
    let result = top().run_with(&mut script);
    assert_eq!(result, 44);
    assert_eq!(script.remaining(), 0);
}
