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

Here's a simple example:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// 1. Define your effects
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// 2. Write effectful functions
#[effectful]
fn greet_user() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    format!("Hello, {}!", name)
}

// 3. Implement handlers
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

// 4. Run with different handlers
fn main() {
    // Production: use real I/O
    let result = greet_user()
        .handle(RealConsoleHandler)
        .run();
    
    println!("Result: {}", result);
}
```

> **📁 Working Example**: See [`examples/readme.rs`](algae/examples/readme.rs) for a complete, runnable version of this code including both real and mock handlers.

## 📚 Core Concepts

### Effects as Descriptions

In algae, effects are **descriptions** of what you want to do, not how to do it:

```rust
effect! {
    FileSystem::Read (String) -> Result<String, std::io::Error>;
    FileSystem::Write ((String, String)) -> Result<(), std::io::Error>;
    Database::Query (String) -> Vec<Row>;
    Logger::Info (String) -> ();
}
```

### Effectful Functions

Functions marked with `#[effectful]` can perform effects using `perform!`:

```rust
#[effectful]
fn process_file(filename: String) -> Result<usize, String> {
    let _: () = perform!(Logger::Info(format!("Processing {}", filename)));
    
    let content: Result<String, std::io::Error> = perform!(FileSystem::Read(filename.clone()));
    let content = content.map_err(|e| format!("Read error: {}", e))?;
    
    let rows: Vec<String> = perform!(Database::Query(
        format!("INSERT INTO files (name, content) VALUES ('{}', '{}')", filename, content)
    ));
    
    let _: () = perform!(Logger::Info(format!("Inserted {} rows", rows.len())));
    Ok(rows.len())
}
```

> **📁 Working Example**: See [`examples/advanced.rs`](algae/examples/advanced.rs) for a complete implementation with multiple effect families, error handling, and testing patterns.

### Handlers as Implementations

Handlers implement the actual behavior for effects:

```rust
struct ProductionHandler {
    db_connection: DatabaseConnection,
}

impl Handler<Op> for ProductionHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileSystem(FileSystem::Read(path)) => {
                Box::new(std::fs::read_to_string(path))
            }
            Op::FileSystem(FileSystem::Write((path, content))) => {
                Box::new(std::fs::write(path, content))
            }
            Op::Database(Database::Query(sql)) => {
                Box::new(self.db_connection.execute(sql))
            }
            Op::Logger(Logger::Info(msg)) => {
                log::info!("{}", msg);
                Box::new(())
            }
        }
    }
}
```

### Testing with Mock Handlers

The same code can be tested with mock implementations:

```rust
struct MockHandler {
    files: HashMap<String, String>,
    logs: Vec<String>,
}

impl Handler<Op> for MockHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileSystem(FileSystem::Read(path)) => {
                Box::new(self.files.get(path).cloned()
                    .ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::NotFound, "File not found"
                    )))
            }
            Op::Logger(Logger::Info(msg)) => {
                self.logs.push(msg.clone());
                Box::new(())
            }
            // ... other operations
        }
    }
}

#[test]
fn test_process_file() {
    let mut handler = MockHandler::new();
    handler.files.insert("test.txt".to_string(), "test content".to_string());
    
    let result = process_file("test.txt".to_string())
        .handle(handler)
        .run();
    
    assert!(result.is_ok());
}
```

> **📁 Working Tests**: See the test modules in [`examples/advanced.rs`](algae/examples/advanced.rs) for complete working test examples.

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

### Run All Examples
```bash
# Run all examples at once
for example in readme advanced theory pure console effect_test minimal; do
    echo "=== Running $example ==="
    cargo run --example $example
    echo
done
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

## 📖 Advanced Usage

### Multiple Effect Families

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
}
```

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