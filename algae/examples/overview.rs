#![feature(coroutines, coroutine_trait, yield_expr)]
//! # Algae Overview - Where to Find Examples
//!
//! This example provides a roadmap to all the examples and documentation
//! in the algae library, helping you find the right starting point.

use algae::prelude::*;

// Simple demonstration effect for this overview
effect! {
    Demo::Message (String) -> ();
    Demo::GetNumber -> i32;
}

struct DemoHandler;

impl Handler<Op> for DemoHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Demo(Demo::Message(msg)) => {
                println!("📢 {}", msg);
                Box::new(())
            }
            Op::Demo(Demo::GetNumber) => Box::new(42),
        }
    }
}

#[effectful]
fn demo_computation() -> i32 {
    let _: () = perform!(Demo::Message("Welcome to Algae!".to_string()));
    let number: i32 = perform!(Demo::GetNumber);
    let _: () = perform!(Demo::Message(format!("The answer is {}", number)));
    number
}

fn main() {
    println!("🌱 Algae - Algebraic Effects for Rust");
    println!("=====================================\n");

    // Run a quick demo
    let result = demo_computation().handle(DemoHandler).run();

    println!("Demo result: {}\n", result);

    println!("📚 LEARNING ROADMAP");
    println!("===================\n");

    println!("🚀 NEW TO ALGEBRAIC EFFECTS?");
    println!("   Start here to understand the concepts:");
    println!("   • README.md - Complete introduction and motivation");
    println!("   • examples/readme.rs - Simple, complete working example");
    println!("   • tests/algebraic_laws.rs - Educational guide to the theory\n");

    println!("🛠️  WANT TO SEE PRACTICAL PATTERNS?");
    println!("   Real-world usage examples:");
    println!("   • examples/advanced.rs - Multi-effect app with testing");
    println!("   • examples/console.rs - Interactive I/O with mocking");
    println!("   • examples/pure.rs - State management patterns\n");

    println!("🎓 INTERESTED IN THE THEORY?");
    println!("   Mathematical foundations:");
    println!("   • examples/theory.rs - Theory-to-code mapping");
    println!("   • tests/algebraic_laws.rs - All 12 algebraic laws explained");
    println!("   • README.md section 'Theoretical Foundations'\n");

    println!("🔧 BUILDING SOMETHING?");
    println!("   Implementation guidance:");
    println!("   • algae/src/lib.rs - Core API documentation");
    println!("   • algae-macros/src/lib.rs - Macro documentation");
    println!("   • examples/advanced.rs - Complex handler patterns\n");

    println!("🧪 TESTING AND DEBUGGING?");
    println!("   Testing patterns and tools:");
    println!("   • examples/advanced.rs - Unit test examples");
    println!("   • examples/console.rs - Mock handler patterns");
    println!("   • algae/tests/ - Comprehensive test suite\n");

    println!("⚙️  CURIOUS ABOUT INTERNALS?");
    println!("   Low-level implementation:");
    println!("   • examples/minimal.rs - Raw coroutine mechanics");
    println!("   • examples/effect_test.rs - Basic functionality");
    println!("   • algae/src/lib.rs - Core implementation\n");

    println!("📖 QUICK REFERENCE");
    println!("==================\n");

    println!("RUN EXAMPLES:");
    println!("   cargo run --example readme     # Quick start");
    println!("   cargo run --example advanced   # Complex patterns");
    println!("   cargo run --example theory     # Theory demo");
    println!("   cargo run --example console    # Interactive I/O");
    println!("   cargo run --example pure       # State management");
    println!("   cargo run --example effect_test # Basic test");
    println!("   cargo run --example minimal    # Low-level\n");

    println!("RUN TESTS:");
    println!("   cargo test                     # All tests");
    println!("   cargo test --test algebraic_laws # Theory tests");
    println!("   cargo test --example advanced  # Example tests\n");

    println!("DOCUMENTATION:");
    println!("   cargo doc --open              # API documentation");
    println!("   README.md                     # Complete guide\n");

    println!("🎯 CHOOSE YOUR PATH");
    println!("===================\n");

    println!("→ Just want to get started? Run: cargo run --example readme");
    println!("→ Want to see real patterns? Run: cargo run --example advanced");
    println!("→ Curious about theory? Read: tests/algebraic_laws.rs");
    println!("→ Need API reference? Run: cargo doc --open\n");

    println!("Happy coding with algebraic effects! 🦀✨");
}
