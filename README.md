# Algae - Algebraic Effects for Rust 🦀

[![Crates.io](https://img.shields.io/crates/v/algae.svg)](https://crates.io/crates/algae)
[![Documentation](https://docs.rs/algae/badge.svg)](https://docs.rs/algae)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Algae** is a Rust library that brings the power of algebraic effects to systems programming. It provides a clean, type-safe way to handle side effects in your programs while maintaining composability, testability, and performance.

Algae implements **one-shot (linear) algebraic effects**, where each effect operation receives exactly one response and continuations are not captured for reuse. This design choice prioritizes simplicity, performance, and ease of understanding while covering the vast majority of real-world use cases.

## 🎯 What are Algebraic Effects?

Algebraic effects are a programming paradigm that allows you to separate the **description** of side effects from their **implementation**. Think of them as a more powerful and composable alternative to traditional approaches like dependency injection or the strategy pattern.

### Key Benefits

- **🔄 Composable**: Effects can be combined and nested naturally
- **🧪 Testable**: Easy to mock and test effectful code
- **🎭 Polymorphic**: Same code can run with different implementations
- **🔒 Type-safe**: All effects are statically checked at compile time
- **⚡ Low-cost**: Minimal runtime overhead using efficient Rust coroutines
- **📏 Linear**: One-shot effects ensure predictable, easy-to-reason-about control flow

## 🚀 Quick Start

Add algae to your `Cargo.toml`:

```toml
[dependencies]
algae = "0.1.0"
```

Enable the required nightly features in your `src/main.rs` or `lib.rs`:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
```

Here's a step-by-step example showing both the explicit and convenient approaches:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// 1. Define your effects
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// 2a. Write effectful functions (explicit approach)
fn greet_user_explicit() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        // Print prompt
        {
            let effect = Effect::new(Console::Print("What's your name?".to_string()).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        // Read input
        let name: String = {
            let effect = Effect::new(Console::ReadLine.into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<String>()
        };
        
        format!("Hello, {}!", name)
    })
}

// 2b. Write effectful functions (convenient approach - same behavior!)
#[effectful]
fn greet_user() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    format!("Hello, {}!", name)
}

// 3. Implement handlers (same for both approaches)
struct RealConsoleHandler;

impl Handler<Op> for RealConsoleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("{}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Box::new(input.trim().to_string())
            }
        }
    }
}

// 4. Run with handlers (both functions work identically)
fn main() {
    // Both approaches produce the same result
    let result1 = greet_user_explicit()
        .handle(RealConsoleHandler)
        .run();
    
    let result2 = greet_user()
        .handle(RealConsoleHandler)
        .run();
    
    println!("Explicit result: {}", result1);
    println!("Convenient result: {}", result2);
    // Both print the same thing!
}
```

**Key insight:** The `#[effectful]` macro is pure convenience - it generates exactly the same `Effectful<R, Op>` type and runtime behavior as the explicit approach, but with much cleaner syntax.

> **📁 Working Examples**: 
> - [`examples/readme.rs`](algae/examples/readme.rs) - Complete version of this code with real and mock handlers
> - [`examples/explicit_vs_convenient.rs`](algae/examples/explicit_vs_convenient.rs) - Side-by-side comparison proving both approaches are identical

## 📚 Core Concepts

Understanding algae requires familiarity with several key types and concepts. This section provides a comprehensive guide to all the core library components and how they work together.

### 🎭 Effects as Descriptions

Effects in algae are **descriptions** of what you want to do, not how to do it. They're defined using the `effect!` macro, which generates Rust enums representing your operations:

```rust
effect! {
    // Each line defines an operation: Family::Operation (Parameters) -> ReturnType
    FileSystem::Read (String) -> Result<String, std::io::Error>;
    FileSystem::Write ((String, String)) -> Result<(), std::io::Error>;
    
    Database::Query (String) -> Vec<Row>;
    Database::Execute (String) -> Result<u64, DbError>;
    
    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
}
```

The `effect!` macro generates several types for you:

```rust
// Generated effect family enums
pub enum FileSystem {
    Read(String),
    Write((String, String)),
}

pub enum Database {
    Query(String),
    Execute(String),
}

pub enum Logger {
    Info(String),
    Error(String),
}

// Generated unified operation type
pub enum Op {
    FileSystem(FileSystem),
    Database(Database),
    Logger(Logger),
}

// Generated conversion traits
impl From<FileSystem> for Op { ... }
impl From<Database> for Op { ... }
impl From<Logger> for Op { ... }
```

### 🔧 `Effectful<R, Op>` - Effectful Computations

The `Effectful<R, Op>` struct is the heart of algae. It represents a computation that:
- **May perform effects** of type `Op` during execution
- **Eventually produces a result** of type `R`
- **Can be run with different handlers** for different behaviors

```rust
// Type signature breakdown:
// Effectful<R, Op>
//          │  └── The type of effects this computation can perform
//          └────── The type of result this computation produces

// Example: A computation that performs Console and Math effects and returns an i32
type MyComputation = Effectful<i32, Op>;
```

#### Creating Effectful Computations (The Explicit Way)

Let's first see how to create effectful computations explicitly to understand what's happening under the hood:

```rust
use algae::prelude::*;

// Explicit function that returns Effectful<R, Op>
fn calculate_with_logging_explicit(x: i32, y: i32) -> Effectful<i32, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        // Manually perform Logger::Info effect
        {
            let effect = Effect::new(Logger::Info(format!("Calculating {} + {}", x, y)).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        // Manually perform Math::Add effect
        let result: i32 = {
            let effect = Effect::new(Math::Add((x, y)).into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<i32>()
        };
        
        // Manually perform another Logger::Info effect
        {
            let effect = Effect::new(Logger::Info(format!("Result: {}", result)).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        result
    })
}
```

This explicit approach shows exactly what's happening:
1. **Return type is explicit**: `Effectful<i32, Op>` - no magic
2. **Coroutine creation**: We manually create the coroutine with `Effectful::new()`
3. **Effect operations**: Each effect is manually created, yielded, and the reply extracted
4. **Type safety**: We explicitly specify the expected return types

#### Creating Effectful Computations (The Convenient Way)

Writing coroutines manually is verbose and error-prone. The `#[effectful]` attribute and `perform!` macro automate this boilerplate:

```rust
#[effectful]
fn calculate_with_logging(x: i32, y: i32) -> i32 {
    let _: () = perform!(Logger::Info(format!("Calculating {} + {}", x, y)));
    let result: i32 = perform!(Math::Add((x, y)));
    let _: () = perform!(Logger::Info(format!("Result: {}", result)));
    result
}
// Actually returns: Effectful<i32, Op> (macro transforms the return type)
```

**What the `#[effectful]` macro does:**
1. **Transforms return type**: `i32` → `Effectful<i32, Op>`
2. **Wraps function body**: Creates the coroutine automatically
3. **Enables `perform!`**: Lets you use the convenient effect syntax

**What the `perform!` macro does:**
1. **Creates the effect**: `Effect::new(operation.into())`
2. **Yields to handler**: `yield effect`
3. **Extracts the reply**: `reply.unwrap().take::<ExpectedType>()`

#### Why Use `#[effectful]`?

The explicit approach is educational but impractical for real code:

| **Explicit Approach** | **`#[effectful]` Approach** |
|----------------------|---------------------------|
| ❌ **Verbose**: 7 lines per effect | ✅ **Concise**: 1 line per effect |
| ❌ **Error-prone**: Manual type annotations | ✅ **Safe**: Automatic type inference |
| ❌ **Repetitive**: Same pattern every time | ✅ **DRY**: Macro handles boilerplate |
| ❌ **Hard to read**: Focus on mechanics | ✅ **Clear intent**: Focus on business logic |
| ✅ **Educational**: Shows what's happening | ✅ **Productive**: Gets work done |

**Equivalence guarantee:** Both approaches produce identical `Effectful<R, Op>` values and have the same runtime behavior.

#### Running Effectful Computations

`Effectful<R, Op>` provides methods for execution:

```rust
let computation = calculate_with_logging(5, 3);

// Method 1: Direct execution with handler
let result: i32 = computation.run_with(MyHandler::new());

// Method 2: Fluent API (recommended)
let result: i32 = computation
    .handle(MyHandler::new())  // Returns Handled<i32, Op, MyHandler>
    .run();                    // Returns i32
```

### 🛠️ `Handler<Op>` - Effect Implementations

The `Handler<Op>` trait defines how effects are actually executed. Handlers are the "interpreters" that give meaning to your effect descriptions:

```rust
pub trait Handler<Op> {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send>;
}
```

#### Type-Safe Effect Handling

Although the return type is type-erased (`Box<dyn Any + Send>`), algae ensures type safety through the effect system:

```rust
struct MyHandler {
    log_count: usize,
}

impl Handler<Op> for MyHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Each branch must return the type specified in the effect! declaration
            Op::Logger(Logger::Info(msg)) => {
                println!("INFO: {}", msg);
                self.log_count += 1;
                Box::new(())  // Must return () as declared
            }
            Op::Math(Math::Add((a, b))) => {
                Box::new(a + b)  // Must return i32 as declared
            }
            Op::FileSystem(FileSystem::Read(path)) => {
                Box::new(std::fs::read_to_string(path))  // Must return Result<String, std::io::Error>
            }
        }
    }
}
```

#### Handler Patterns

**Production Handler:**
```rust
struct ProductionHandler {
    db_pool: ConnectionPool,
    logger: Logger,
}

impl Handler<Op> for ProductionHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Database(Database::Query(sql)) => {
                let rows = self.db_pool.execute(sql).unwrap();
                Box::new(rows)
            }
            Op::Logger(Logger::Info(msg)) => {
                self.logger.info(msg);
                Box::new(())
            }
        }
    }
}
```

**Test Handler:**
```rust
struct MockHandler {
    db_responses: HashMap<String, Vec<Row>>,
    logged_messages: Vec<String>,
}

impl Handler<Op> for MockHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Database(Database::Query(sql)) => {
                let rows = self.db_responses.get(sql).cloned().unwrap_or_default();
                Box::new(rows)
            }
            Op::Logger(Logger::Info(msg)) => {
                self.logged_messages.push(msg.clone());
                Box::new(())
            }
        }
    }
}
```

### ⚡ `Effect<Op>` and `Reply` - The Runtime Types

These are the low-level types that power the effect system. You typically don't use them directly, but understanding them helps you understand how algae works internally.

#### `Effect<Op>` - Effect Requests

An `Effect<Op>` represents a single effect operation that has been requested but not yet handled:

```rust
pub struct Effect<Op> {
    pub op: Op,                           // The operation being requested
    reply: Option<Box<dyn Any + Send>>,   // Storage for the handler's response
}
```

```rust
// Created automatically by perform!() macro
let effect = Effect::new(Logger::Info("Hello".to_string()));

// Handler fills the effect with a response
effect.fill_boxed(Box::new(()));

// Extract the response
let reply = effect.get_reply();
```

#### `Reply` - Typed Response Extraction

A `Reply` wraps the handler's response and provides type-safe extraction:

```rust
pub struct Reply {
    value: Box<dyn Any + Send>,  // Type-erased response from handler
}

impl Reply {
    pub fn take<R: Any + Send>(self) -> R {
        // Runtime type checking + extraction
        // Panics if types don't match
    }
}
```

```rust
// Created when extracting from Effect
let reply: Reply = effect.get_reply();

// Type-safe extraction (must match effect declaration)
let response: () = reply.take::<()>();  // For Logger::Info -> ()
let result: i32 = reply.take::<i32>();   // For Math::Add -> i32
```

### 🔄 The Execution Model

Understanding how algae executes effectful computations helps you write better code and debug issues:

#### 1. Compilation Phase

```rust
// What you write with the convenient syntax:
#[effectful]
fn my_function() -> String {
    let value: i32 = perform!(Math::Add((2, 3)));
    format!("Result: {}", value)
}

// What the macros generate (equivalent to explicit approach):
fn my_function() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        // perform!(Math::Add((2, 3))) expands to:
        let value: i32 = {
            let __eff = Effect::new(Math::Add((2, 3)).into());
            let __reply_opt = yield __eff;
            __reply_opt.unwrap().take::<i32>()
        };
        format!("Result: {}", value)
    })
}

// This is identical to what you'd write explicitly:
fn my_function_explicit() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        let value: i32 = {
            let effect = Effect::new(Math::Add((2, 3)).into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<i32>()
        };
        format!("Result: {}", value)
    })
}
```

#### 2. Execution Phase

```rust
let computation = my_function();
let result = computation.handle(MyHandler::new()).run();
```

**Step-by-step execution:**

1. **Start coroutine** with `None` (no previous reply)
2. **Hit `perform!`** - creates `Effect::new(Math::Add((2, 3)))`
3. **Yield effect** to handler and suspend coroutine
4. **Handler processes** `Math::Add((2, 3))` and returns `Box::new(5i32)`
5. **Fill effect** with handler's response
6. **Resume coroutine** with `Some(Reply { value: Box::new(5i32) })`
7. **Extract result** using `reply.take::<i32>()` → `5i32`
8. **Continue execution** with the extracted value
9. **Return final result** `"Result: 5"`

#### 3. Type Safety at Runtime

```rust
// Effect declaration says Math::Add returns i32
effect! {
    Math::Add ((i32, i32)) -> i32;
}

// Handler must return i32 (but as Box<dyn Any + Send>)
impl Handler<Op> for MyHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Math(Math::Add((a, b))) => Box::new(a + b), // ✅ Returns i32
            // Op::Math(Math::Add((a, b))) => Box::new("hello"), // ❌ Would panic at runtime
        }
    }
}

// perform! expects i32 (enforced at runtime)
let value: i32 = perform!(Math::Add((2, 3))); // ✅ Type matches
// let value: String = perform!(Math::Add((2, 3))); // ❌ Would panic at runtime
```

### 🔗 Type Relationships

Here's how all the types work together:

```rust
// 1. Effect declaration generates operation types
effect! {
    Console::Print (String) -> ();
    Math::Add ((i32, i32)) -> i32;
}
// Generates: Console, Math, Op enums + From impls

// 2. Effectful functions return Effectful<R, Op>
#[effectful]
fn interactive_calculator() -> i32 {           // Returns Effectful<i32, Op>
    let _: () = perform!(Console::Print("Enter numbers...".to_string()));
    let result: i32 = perform!(Math::Add((5, 3)));
    result
}

// 3. Handlers implement behavior for Op
struct MyHandler;
impl Handler<Op> for MyHandler { ... }

// 4. Execution ties everything together
let computation: Effectful<i32, Op> = interactive_calculator();
let handled: Handled<i32, Op, MyHandler> = computation.handle(MyHandler);
let result: i32 = handled.run();
```

### 🧪 Testing Patterns

The type system makes testing effectful code straightforward:

```rust
#[effectful]
fn user_workflow() -> String {
    let _: () = perform!(Logger::Info("Starting workflow".to_string()));
    let name: String = perform!(Console::ReadLine);
    let _: () = perform!(Logger::Info(format!("Hello, {}", name)));
    name
}

#[test]
fn test_user_workflow() {
    struct TestHandler {
        input: String,
        logs: Vec<String>,
    }
    
    impl Handler<Op> for TestHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            match op {
                Op::Console(Console::ReadLine) => Box::new(self.input.clone()),
                Op::Logger(Logger::Info(msg)) => {
                    self.logs.push(msg.clone());
                    Box::new(())
                }
            }
        }
    }
    
    let mut handler = TestHandler {
        input: "Alice".to_string(),
        logs: Vec::new(),
    };
    
    let result = user_workflow().handle(handler).run();
    assert_eq!(result, "Alice");
    // handler.logs contains the logged messages
}
```

This comprehensive type system ensures that:
- **Effects are declared once** and used consistently
- **Handlers provide correct return types** (checked at runtime)
- **Effectful functions get properly typed results** from effects
- **Testing is straightforward** with mock handlers
- **Composition is natural** through the trait system

## 🔗 Theoretical Foundations

Algae is based on the mathematical theory of algebraic effects and handlers, developed by researchers like Gordon Plotkin and Matija Pretnar.

### One-Shot vs Multi-Shot Effects

Algae implements **one-shot (linear) algebraic effects**. Understanding this design choice helps explain what algae can and cannot do:

#### One-Shot Effects (What Algae Implements)

- **Single Response**: Each effect operation receives exactly one response
- **No Continuation Capture**: Computation state is not saved for later reuse
- **Linear Control Flow**: Effects execute once and continue forward
- **Simpler Implementation**: Easier to understand, debug, and optimize
- **Better Performance**: No overhead from capturing and managing continuations

```rust
// ✅ Supported: Traditional side effects
perform!(File::Read("config.txt"))     // Read once, get result once
perform!(Database::Query("SELECT...")) // Query once, get rows once
perform!(Logger::Info("Starting..."))  // Log once, acknowledge once
```

#### Multi-Shot Effects (What Algae Does NOT Implement)

- **Multiple Responses**: Effect operations can be resumed multiple times
- **Continuation Capture**: Computation state is captured and reusable
- **Non-Linear Control Flow**: Effects can branch, backtrack, or iterate
- **Complex Implementation**: Requires sophisticated continuation management
- **Higher Overhead**: Performance cost of capturing and managing state

```rust
// ❌ Not supported: Non-deterministic, generator-style effects
perform!(Choice::Select(vec![1,2,3]))  // Cannot try all options
perform!(Generator::Yield(value))      // Cannot yield multiple values
perform!(Search::Backtrack)            // Cannot rewind and try alternatives
```

#### Why One-Shot?

1. **Covers 90% of Use Cases**: File I/O, networking, databases, logging, state management
2. **Easier to Learn**: Simpler mental model for developers new to algebraic effects
3. **Better Performance**: No continuation overhead means faster execution
4. **Reliable**: Fewer edge cases and potential for subtle bugs
5. **Rust-Friendly**: Aligns well with Rust's ownership model and zero-cost abstractions

For advanced use cases requiring multi-shot effects (like probabilistic programming, 
non-deterministic search, or complex generators), consider specialized libraries or
implementing custom continuation-passing patterns.

### Mapping to Theory

| **Theory** | **Algae Implementation** | **Purpose** |
|------------|-------------------------|-------------|
| **Effect Signature** | `effect!` macro | Declares operations and their types |
| **Effect Operation** | `perform!(Operation)` | Invokes an effect operation |
| **Handler** | `Handler<Op>` trait | Provides interpretation for operations |
| **Handled Computation** | `Effectful<R, Op>` | Computation that may perform effects |
| **Handler Installation** | `.handle(h).run()` | Applies handler to computation |

> **📁 Theory in Practice**: See [`examples/theory.rs`](algae/examples/theory.rs) for a complete demonstration of how these theoretical concepts map to working code.

### Algebraic Laws

Algae respects the fundamental algebraic laws of effects:

1. **Associativity**: `(a >> b) >> c ≡ a >> (b >> c)`
2. **Identity**: Handler for no-op effects acts as identity
3. **Homomorphism**: Handlers preserve the algebraic structure

> **📁 Laws in Action**: See [`tests/algebraic_laws.rs`](algae/tests/algebraic_laws.rs) for comprehensive tests and educational explanations of all 12 algebraic laws, including beginner-friendly introductions to the mathematical concepts.

### Comparison with Other Approaches

| **Approach** | **Composability** | **Type Safety** | **Performance** | **Testability** |
|--------------|-------------------|-----------------|-----------------|------------------|
| **Algebraic Effects** | ✅ Excellent | ✅ Full | ✅ Low-cost | ✅ Excellent |
| **Async/Await** | ⚠️ Limited | ✅ Good | ✅ Good | ⚠️ Moderate |
| **Dependency Injection** | ⚠️ Moderate | ⚠️ Runtime | ⚠️ Overhead | ✅ Good |
| **Global State** | ❌ Poor | ❌ None | ✅ Fast | ❌ Poor |

## 🏗️ Architecture

### Library Structure

```
algae/
├── algae/                 # Core library
│   ├── src/lib.rs        # Effect, Effectful, Handler types
│   └── examples/         # Example programs
├── algae-macros/         # Procedural macros
│   └── src/lib.rs        # effect!, #[effectful], perform! macros
└── README.md
```

### Generated Code

The `effect!` macro generates:

```rust
// From this:
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// Generates this:
#[derive(Debug, Clone)]
pub enum Console {
    Print(String),
    ReadLine,
}

#[derive(Debug, Clone)]
pub enum Op {
    Console(Console),
}

impl From<Console> for Op {
    fn from(c: Console) -> Op { Op::Console(c) }
}
```

### Runtime Behavior

1. **Effectful Function Call**: Returns `Effectful<R, Op>` (zero-cost wrapper)
2. **Handler Installation**: Creates `Handled<R, Op, H>` (zero-cost wrapper)
3. **Execution**: Drives coroutine, yielding effects to handler
4. **Effect Processing**: Handler processes operation, returns typed result
5. **Resume**: Coroutine resumes with handler's reply

## 🧪 Examples

The library includes several examples demonstrating different patterns:

### Getting Started Guide
```bash
cargo run --example overview
```
Comprehensive roadmap showing where to find all examples, tests, and documentation.

### Quick Start - README Example
```bash
cargo run --example readme
```
Complete, runnable version of the README's introductory example with both real and mock handlers.

### Explicit vs Convenient Syntax
```bash
cargo run --example explicit_vs_convenient
```
Side-by-side demonstration showing that `#[effectful]` and `perform!` are pure convenience macros that generate identical code to the explicit approach.

### Multiple Effects Patterns
```bash
cargo run --example multiple_effects_demo
```
Comprehensive guide to organizing multiple effects: single declaration vs module separation, with trade-offs and best practices.

### Custom Root Effects  
```bash
cargo run --example custom_root_effects
```
Demonstrates the new custom root enum functionality: avoiding conflicts, combining roots, and managing multiple effect declarations in one module.

### Advanced Patterns
```bash
cargo run --example advanced
```
Complex multi-effect application with file I/O, database operations, logging, error handling, and comprehensive testing patterns.

### Theoretical Foundations
```bash
cargo run --example theory
```
Demonstrates the mapping between algebraic effects theory and algae implementation, including algebraic laws.

### State Management
```bash
cargo run --example pure
```
Shows pure functional state management using algebraic effects.

### Interactive I/O
```bash
cargo run --example console
```
Demonstrates interactive I/O with both real and mock implementations, plus random number generation.

### Basic Functionality
```bash
cargo run --example effect_test
```
Basic test of the effect system with simple operations.

### Low-Level Coroutines
```bash
cargo run --example minimal
```
Minimal example showing the underlying coroutine mechanics (educational).

### No-Macros Usage
```bash
cargo run --example no_macros --no-default-features
```
Complete example showing how to use algae without any macros - pure explicit syntax.

### Run All Examples
```bash
# Run all examples at once
for example in readme explicit_vs_convenient multiple_effects_demo advanced theory pure console effect_test minimal; do
    echo "=== Running $example ==="
    cargo run --example $example
    echo
done

# Run no-macros example separately (requires different feature flags)
echo "=== Running no_macros ==="
cargo run --example no_macros --no-default-features
```

## 🔧 Development

### Prerequisites

- **Rust Nightly**: Required for coroutine features
- **Git**: For cloning the repository

### Setup

```bash
# Clone the repository
git clone https://github.com/your-username/algae.git
cd algae

# Ensure you're using nightly Rust
rustup default nightly

# Or set up a toolchain file (already included)
cat rust-toolchain.toml
```

### Building

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Build documentation
cargo doc --open
```

### Testing

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests  
cargo test --test '*'

# Run only documentation tests
cargo test --doc

# Run with verbose output
cargo test -- --nocapture
```

### Code Quality

```bash
# Check for issues
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Examples

```bash
# Run all examples
cargo run --example pure
cargo run --example console  
cargo run --example debug
cargo run --example effect_test

# Run specific example with release optimizations
cargo run --release --example console
```

### Benchmarking

```bash
# Run benchmarks (if implemented)
cargo bench

# Profile memory usage
cargo run --example pure --features profiling
```

## 🎛️ Optional Macros Feature

Algae's macros (`effect!`, `#[effectful]`, `perform!`) are **optional**. You can disable them if you prefer explicit syntax or have restrictions on proc-macros.

### Default: Macros Enabled

```toml
[dependencies]
algae = "0.1.0"  # macros feature enabled by default
```

### Disable Macros

```toml
[dependencies]
algae = { version = "0.1.0", default-features = false }
```

### No-Macros Example

When macros are disabled, you define everything manually:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;  // Only exports core types, no macros
use std::any::Any;

// 1. Manually define effect enums (instead of effect! macro)
#[derive(Debug)]
pub enum Console {
    Print(String),
    ReadLine,
}

#[derive(Debug)]  
pub enum Op {
    Console(Console),
}

impl From<Console> for Op {
    fn from(c: Console) -> Self {
        Op::Console(c)
    }
}

// 2. Manually create effectful functions (instead of #[effectful])
fn greet_user() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] |mut _reply: Option<Reply>| {
        // Manual effect operations (instead of perform!)
        {
            let effect = Effect::new(Console::Print("What's your name?".to_string()).into());
            let reply_opt = yield effect;
            let _: () = reply_opt.unwrap().take::<()>();
        }
        
        let name: String = {
            let effect = Effect::new(Console::ReadLine.into());
            let reply_opt = yield effect;
            reply_opt.unwrap().take::<String>()
        };
        
        format!("Hello, {}!", name)
    })
}

// 3. Handlers work exactly the same
struct ConsoleHandler;
impl Handler<Op> for ConsoleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("{}", msg);
                Box::new(())
            }
            Op::Console(Console::ReadLine) => {
                // In real code, read from stdin
                Box::new("Alice".to_string())
            }
        }
    }
}

// 4. Execution is identical
fn main() {
    let result = greet_user()
        .handle(ConsoleHandler)
        .run();
    println!("Result: {}", result);
}
```

> **📁 Working Example**: See [`examples/no_macros.rs`](algae/examples/no_macros.rs) for a complete working example without macros.

### When to Disable Macros

**Use the manual approach when:**
- **Proc-macro restrictions**: Your environment doesn't allow procedural macros
- **Full control**: You need custom implementations of the generated types
- **Library development**: Minimizing dependencies for a library crate
- **Learning**: Understanding exactly how the effects system works
- **Custom syntax**: Building your own effect DSL on top of algae

**Use macros (default) when:**
- **Productivity**: You want clean, readable application code
- **Rapid development**: Prototyping or building applications quickly
- **Standard use cases**: The generated code meets your needs
- **Team development**: Consistent, familiar syntax for all developers

### Feature Compatibility

Both approaches provide identical capabilities:
- ✅ **One-shot algebraic effects** - Same runtime model
- ✅ **Type-safe effect handlers** - Same type system
- ✅ **Composable effect systems** - Same composition patterns
- ✅ **Zero-cost abstractions** - Same performance characteristics
- ✅ **Full coroutine support** - Same underlying implementation

**The only difference is syntax for defining and using effects.**

## 📖 Advanced Usage

### Multiple Effect Families

#### ✅ Recommended: Single `effect!` Declaration

You can define multiple effect families in a single declaration:

```rust
effect! {
    // File operations
    File::Read (String) -> Result<String, std::io::Error>;
    File::Write ((String, String)) -> Result<(), std::io::Error>;
    
    // Network operations  
    Http::Get (String) -> Result<String, reqwest::Error>;
    Http::Post ((String, String)) -> Result<String, reqwest::Error>;
    
    // Database operations
    Db::Query (String) -> Vec<Row>;
    Db::Execute (String) -> Result<u64, DbError>;
    
    // Logging operations
    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
}
```

This generates a single `Op` enum that contains all your effect families:

```rust
// Generated by the macro
pub enum Op {
    File(File),
    Http(Http), 
    Db(Db),
    Logger(Logger),
}
```

#### ✅ New: Custom Root Enum Names

You can now use multiple `effect!` declarations in the same module by specifying custom root enum names:

```rust
// ✅ Works with custom root names
effect! {
    root ConsoleOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

effect! {
    root FileOp;
    File::Read (String) -> Result<String, String>;
    File::Write ((String, String)) -> Result<(), String>;
}

effect! {
    root NetworkOp;
    Http::Get (String) -> Result<String, String>;
    Http::Post ((String, String)) -> Result<String, String>;
}
```

Each generates its own root enum:
- `ConsoleOp` containing `Console` variants
- `FileOp` containing `File` variants  
- `NetworkOp` containing `Http` variants

You can then combine them using the `combine_roots!` macro:

```rust
// Combine multiple root enums into one
algae::combine_roots!(pub Op = ConsoleOp, FileOp, NetworkOp);

// Now you can write unified handlers
impl Handler<Op> for UnifiedHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::ConsoleOp(console_op) => self.console_handler.handle(console_op),
            Op::FileOp(file_op) => self.file_handler.handle(file_op),
            Op::NetworkOp(network_op) => self.network_handler.handle(network_op),
        }
    }
}
```

#### ❌ Error Detection: Duplicate Root Names

Attempting to use duplicate root names (including the default `Op`) in the same scope will produce clear error messages:

```rust
// ❌ ERROR: Conflicting Op enum definitions
effect! {
    Console::Print (String) -> ();
}

effect! {
    Math::Add ((i32, i32)) -> i32;
}
// Error: duplicate definition of `Op`
```

Each `effect!` macro generates its own `Op` enum, so multiple declarations in the same scope create conflicting type definitions.

#### ✅ Alternative: Module-Based Separation

For large codebases, you can separate effects into modules:

```rust
mod console_effects {
    use algae::prelude::*;
    
    effect! {
        Console::Print (String) -> ();
        Console::ReadLine -> String;
    }
    
    #[effectful]
    pub fn interactive_session() -> String {
        let _: () = perform!(Console::Print("Hello!".to_string()));
        let name: String = perform!(Console::ReadLine);
        name
    }
}

mod math_effects {
    use algae::prelude::*;
    
    effect! {
        Math::Add ((i32, i32)) -> i32;
        Math::Multiply ((i32, i32)) -> i32;
    }
    
    #[effectful] 
    pub fn calculation(x: i32, y: i32) -> i32 {
        let sum: i32 = perform!(Math::Add((x, y)));
        perform!(Math::Multiply((sum, 2)))
    }
}
```

**Trade-offs of module separation:**
- ✅ **Good for**: Large teams, feature boundaries, independent testing
- ❌ **Limitation**: Can't easily compose effects across modules
- ❌ **Complexity**: Each module needs its own handler

#### 📁 When to Use Each Approach

| **Single `effect!`** | **Module Separation** |
|---------------------|----------------------|
| ✅ Small to medium projects | ✅ Large codebases with teams |
| ✅ Effects that interact | ✅ Independent feature areas |
| ✅ Single unified handler | ✅ Separate testing strategies |
| ✅ Easy composition | ❌ Complex cross-module composition |

> **📁 Working Example**: See [`examples/multiple_effects_demo.rs`](algae/examples/multiple_effects_demo.rs) for complete demonstrations of both patterns.

### Handler Composition

Handlers can be composed to handle different effect families:

```rust
struct CompositeHandler {
    file_handler: FileHandler,
    http_handler: HttpHandler, 
    db_handler: DbHandler,
}

impl Handler<Op> for CompositeHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::File(_) => self.file_handler.handle(op),
            Op::Http(_) => self.http_handler.handle(op),
            Op::Db(_) => self.db_handler.handle(op),
        }
    }
}
```

### Error Handling Patterns

Effects naturally support `Result` types for error handling:

```rust
#[effectful]
fn safe_file_operation(path: String) -> Result<String, AppError> {
    let content: Result<String, std::io::Error> = perform!(File::Read(path.clone()));
    let content = content.map_err(AppError::IoError)?;
    
    let result: Result<(), std::io::Error> = perform!(File::Write((
        format!("{}.backup", path),
        content.clone()
    )));
    result.map_err(AppError::IoError)?;
    
    Ok(content)
}
```

### Control Flow

Effectful functions support all Rust control flow:

```rust
#[effectful]
fn batch_process(items: Vec<String>) -> Vec<Result<String, String>> {
    let mut results = Vec::new();
    
    for (i, item) in items.iter().enumerate() {
        let _: () = perform!(Logger::Info(format!("Processing item {}: {}", i, item)));
        
        let result = match item.as_str() {
            "skip" => {
                let _: () = perform!(Logger::Info("Skipping item".to_string()));
                continue;
            }
            "break" => {
                let _: () = perform!(Logger::Info("Breaking early".to_string())); 
                break;
            }
            _ => {
                let processed: Result<String, String> = perform!(Processor::Handle(item.clone()));
                processed
            }
        };
        
        results.push(result);
    }
    
    results
}
```

## 🔬 Performance

### Benchmarks

Algae is designed for minimal runtime overhead:

- **Effect Declaration**: Compile-time only, no runtime cost
- **Effectful Functions**: Single heap allocation for coroutine state machine
- **Handler Calls**: Static dispatch with dynamic typing for return values
- **Type Safety**: Compile-time checked effects, runtime type verification for replies
- **Performance Cost**: Comparable to `async/await` but with more flexibility

### Memory Usage

- **Single Allocation per Computation**: One heap allocation for the coroutine state
- **Stack-Safe**: Uses coroutines instead of recursion for deep effect chains
- **No GC Pressure**: All allocations are explicit and bounded
- **Dynamic Typing Overhead**: `Box<dyn Any + Send>` for handler return values

### Performance Considerations

**Costs:**
- One heap allocation per effectful computation (for coroutine state)
- Dynamic type checking when extracting handler replies (`Reply::take()`)
- Coroutine suspend/resume overhead (similar to async/await)
- Pattern matching on effect operations

**Optimizations:**
1. **Minimize effect frequency**: Batch operations when possible
2. **Use concrete handler types**: Avoid trait objects where possible  
3. **Profile critical paths**: Effects add overhead to hot loops
4. **Consider alternatives**: For tight loops, direct function calls may be faster

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test`
6. Run clippy: `cargo clippy --all-targets -- -D warnings`
7. Format your code: `cargo fmt`
8. Submit a pull request

### Areas for Contribution

- **Documentation**: Improve examples and guides
- **Performance**: Benchmarks and optimizations
- **Testing**: Additional test cases and property tests
- **Examples**: Real-world usage examples
- **Integrations**: Async/await compatibility, tokio integration

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Gordon Plotkin** and **Matija Pretnar** for the theoretical foundations of algebraic effects
- **The Rust Community** for excellent tools and ecosystem
- **OCaml's Effects** for inspiration on practical algebraic effects
- **Koka Language** for demonstrating effect types in systems programming
- **Eff Language** for the original algebraic effects implementation

## 📚 Further Reading

### Academic Papers
- [Algebraic Effects and Handlers](https://www.eff-lang.org/handlers-tutorial.pdf) - Tutorial introduction
- [An Introduction to Algebraic Effects and Handlers](https://www.cs.ox.ac.uk/people/jeremy.gibbons/publications/handlers.pdf) - Comprehensive overview
- [Handling Asynchronous Exceptions with Algebraic Effects](https://arxiv.org/abs/1310.3981) - Advanced applications

### Other Implementations
- [Eff Language](https://www.eff-lang.org/) - The original algebraic effects language
- [Koka](https://koka-lang.github.io/) - Microsoft's research language with effect types  
- [OCaml 5.0 Effects](https://ocaml.org/manual/effects.html) - Effects in OCaml
- [Unison](https://www.unison-lang.org/) - Functional language with algebraic effects

### Blog Posts and Tutorials
- [Algebraic Effects for the Rest of Us](https://overreacted.io/algebraic-effects-for-the-rest-of-us/) - Accessible introduction
- [Effects in Rust](https://boats.gitlab.io/blog/post/await-decision/) - Rust-specific discussions
- [What are Algebraic Effects?](https://jrsinclair.com/articles/2019/algebraic-effects-what-are-they/) - Practical explanation

---

**Built with ❤️ and Rust 🦀**