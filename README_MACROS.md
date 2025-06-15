# Using Algae With and Without Macros

Algae provides convenient macros by default, but they can be disabled if you prefer a more explicit approach or have restrictions on proc-macros.

## Default Usage (With Macros)

By default, algae includes the `macros` feature which provides the `effect!`, `#[effectful]`, and `perform!` macros:

```toml
[dependencies]
algae = "0.1.0"  # macros feature enabled by default
```

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define effects using the effect! macro
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// Create effectful functions using the #[effectful] attribute
#[effectful]
fn greet_user() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    format!("Hello, {}!", name)
}
```

## Usage Without Macros

To disable macros, exclude the default features:

```toml
[dependencies]
algae = { version = "0.1.0", default-features = false }
```

Then define your effects and handlers manually:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;
use std::any::Any;

// Manually define effect operations
#[derive(Debug)]
pub enum Console {
    Print(String),
    ReadLine,
}

impl Default for Console {
    fn default() -> Self {
        Console::ReadLine
    }
}

#[derive(Debug)]
pub enum Op {
    Console(Console),
}

impl Default for Op {
    fn default() -> Self {
        Op::Console(Console::default())
    }
}

impl From<Console> for Op {
    fn from(c: Console) -> Self {
        Op::Console(c)
    }
}

// Manually create effectful computation
fn greet_user() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] |mut _reply: Option<Reply>| {
        // Equivalent to perform!(Console::Print(...))
        {
            let __eff = Effect::new(Console::Print("What's your name?".to_string()).into());
            let __reply_opt = yield __eff;
            let _: () = __reply_opt.unwrap().take::<()>();
        }
        
        // Equivalent to perform!(Console::ReadLine)
        let name: String = {
            let __eff = Effect::new(Console::ReadLine.into());
            let __reply_opt = yield __eff;
            __reply_opt.unwrap().take::<String>()
        };
        
        format!("Hello, {}!", name)
    })
}
```

## When to Use Each Approach

### Use Macros (Default) When:
- You want clean, readable syntax
- You're prototyping or building applications quickly
- Proc-macros are allowed in your environment
- You prefer declarative effect definitions

### Use Manual Approach When:
- You need full control over generated code
- Proc-macros are restricted in your environment
- You're building a library that should minimize dependencies
- You want to understand exactly how the effects system works
- You need custom implementations of the generated types

## Feature Compatibility

Both approaches provide the same runtime capabilities:
- One-shot algebraic effects
- Type-safe effect handlers
- Composable effect systems
- Zero-cost abstractions
- Full coroutine support

The only difference is the syntax for defining and using effects.