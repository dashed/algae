# Algae — Algebraic Effects for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Experimental toy project.** Built for exploring algebraic effects in Rust.
> It depends on unstable nightly coroutine features, the API changes without
> notice, and it is not published to crates.io. Do not use it in production.

Algae implements **one-shot (linear) algebraic effects** on top of Rust's native
coroutines. You describe the operations your code performs (`effect!`), write
functions that request them (`#[effectful]` + `perform!`), and supply the actual
behavior separately as a handler. The same function can run against real I/O in
production and an in-memory mock in tests, without trait objects or dependency
injection plumbing.

"One-shot" means each performed operation receives exactly one reply and the
computation continues forward; continuations are never captured for replay. This
covers ordinary side effects (I/O, state, logging, error reporting) and keeps
the implementation small. Multi-shot effects (non-determinism, backtracking,
generators) are out of scope.

## Quick start

Algae is not on crates.io; use it as a git or path dependency:

```toml
[dependencies]
algae = { git = "https://github.com/dashed/algae.git" }
```

You need nightly Rust, and any crate that uses `#[effectful]`/`perform!` (or
writes the coroutines by hand) must enable the coroutine features —
`effect!` alone needs none:

```rust
#![feature(coroutines, yield_expr)]
```

(`coroutine_trait` is only needed if you name the `Coroutine` trait yourself,
e.g. to write a custom driver.)

A complete program:

```rust
#![feature(coroutines, yield_expr)]
use algae::prelude::*;

// 1. Declare operations: Family::Operation (Payload) -> ReturnType
effect! {
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

// 2. Write effectful functions. #[effectful] turns the return type into
//    Effectful<String, Op>; perform! requests an operation and yields
//    until a handler replies.
#[effectful]
fn greet_user() -> String {
    let _: () = perform!(Console::Print("What's your name?".to_string()));
    let name: String = perform!(Console::ReadLine);
    format!("Hello, {name}!")
}

// 3. Implement the behavior. Each match arm must return the type declared
//    in effect! (boxed); a mismatch panics at extraction with a message
//    naming both types.
struct RealConsoleHandler;

impl Handler<Op> for RealConsoleHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::Console(Console::Print(msg)) => {
                println!("{msg}");
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

// 4. Run.
fn main() {
    let greeting = greet_user().handle(RealConsoleHandler).run();
    println!("{greeting}");
}
```

Testing the same function means swapping the handler — no changes to
`greet_user`. See [`examples/readme.rs`](algae/examples/readme.rs) for this
program with both a real and a mock console handler.

## How it works

`#[effectful]` and `perform!` are thin sugar over a coroutine. This function:

```rust
#[effectful]
fn my_function() -> String {
    let value: i32 = perform!(Math::Add((2, 3)));
    format!("Result: {value}")
}
```

expands to roughly:

```rust
fn my_function() -> Effectful<String, Op> {
    Effectful::new(#[coroutine] move |mut _reply: Option<Reply>| {
        let value: i32 = {
            let __eff = Effect::new(Math::Add((2, 3)).into());
            let __reply_opt = yield __eff;
            __reply_opt.expect("...").take::<i32>()
        };
        format!("Result: {value}")
    })
}
```

Running a computation drives the coroutine: each yielded `Effect<Op>` is passed
to the handler, the boxed reply is handed back through `Reply`, and the
coroutine resumes. `Reply::take::<T>()` checks the concrete type at runtime and
panics with a descriptive message on mismatch; `try_take` is the non-panicking
variant.

The macros are optional. With `default-features = false` you write the enums
and coroutines by hand against the same runtime —
[`examples/no_macros.rs`](algae/examples/no_macros.rs) shows the full pattern,
and [`examples/explicit_vs_convenient.rs`](algae/examples/explicit_vs_convenient.rs)
shows both styles side by side.

### Generated code and payload requirements

For each `effect!` block the macro generates one enum per family, a root enum
(named `Op` by default) with a variant per family, and `From<Family> for Root`
impls. All generated enums are `pub` and derive `Debug`, `Clone`, and
`PartialEq` — so every payload type must implement those three traits.

## The typed API

Alongside the enums, `effect!` generates a **typed API** that removes both the
annotation tax at perform sites and the boxing/matching boilerplate in
handlers.

Every operation gets a snake_case constructor carrying its declared reply
type. `perform!` accepts these and infers everything — and asking for the
wrong type becomes a *compile* error at the perform site instead of a runtime
panic:

```rust
#[effectful]
fn greet() -> String {
    perform!(Console::print("What's your name?".to_string())); // no `let _: () =`
    let name = perform!(Console::read_line());  // inferred String
    let n = perform!(Math::add(1, 2));          // tuple payload -> named parameters
    format!("Hello, {name}! ({n})")
}
```

Every family gets a typed handler trait plus an adapter that turns any
implementation into a `PartialHandler` — no `Box::new`, no nested `match`,
and a missing operation is a missing-method compile error:

```rust
struct MockConsole { input: String }

impl ConsoleOps for MockConsole {
    fn print(&mut self, value: &String) { println!("[mock] {value}"); }
    fn read_line(&mut self) -> String { self.input.clone() }
}

let result = greet()
    .begin_chain()
    .handle(HandleConsole(MockConsole { input: "Ada".into() }))
    .handle(HandleMath(RealMath))
    .run_checked()?;
```

Naming: `ReadLine` → `read_line` (keywords become raw identifiers: `Move` →
`r#move`); trait and adapter are `{Family}Ops` / `Handle{Family}`. The legacy
forms — `perform!(Console::Print(msg))` with an annotation, hand-written
`Handler` impls — keep working unchanged, in the same functions. See
[`examples/typed_api.rs`](algae/examples/typed_api.rs).

## Handlers

There are two handler traits:

- **`Handler<Op>`** — total: must answer every operation. Run with
  `.handle(h).run()` or `run_with(h)`.
- **`PartialHandler<Op>`** — may decline by returning `None`. Run with
  `run_checked`, which returns `Result<R, UnhandledOp<Op>>` instead of
  panicking. `UnhandledOp` carries the declined operation and implements
  `Display`/`Error`.

Partial handlers compose. `VecHandler` tries handlers in order, and chains can
be built fluently — any `PartialHandler + Send + 'static` value works directly:

```rust
// All at once
let result = program()
    .handle_all(vec![
        Box::new(MathHandler) as Box<dyn PartialHandler<Op> + Send>,
        Box::new(LoggerHandler),
    ])
    .run_checked()?;

// One by one
let result = program()
    .begin_chain()
    .handle(MathHandler)
    .handle(LoggerHandler)
    .run_checked()?;

// Conditionally
let mut handled = program().begin_chain().handle(MathHandler);
if verbose {
    handled = handled.handle(LoggerHandler);
}
let result = handled.run_checked()?;
```

To fold a prebuilt `VecHandler`'s handlers into a chain without nesting, use
`.merge(vec_handler)`. A total `Handler` can join a chain via `handle_total`,
and can be run checked with `run_checked_with`.

Note: `VecHandler` also implements the total `Handler` trait for convenience,
but that path **panics** on an unhandled operation — prefer `run_checked` when
composing partial handlers.

```rust
struct MathHandler;

impl PartialHandler<Op> for MathHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Math(Math::Add((a, b))) => Some(Box::new(a + b)),
            _ => None, // decline everything else
        }
    }
}
```

For inline handlers, `handler_fn(|op| ...)` wraps a closure as a
`PartialHandler` with no struct required.

### Testing helpers

A `&mut` reference to a handler is itself a handler, so tests can keep
ownership of a mock and inspect it after the run:

```rust
let mut mock = RecordingHandler::new();
let result = program().run_with(&mut mock);
assert_eq!(mock.logs, ["started", "finished"]);
```

For tests where the answers are just canned values, `ScriptHandler` serves
replies in declaration order — no handler struct, no `match`:

```rust
let mut script = ScriptHandler::named("console")
    .reply(())                    // answers Console::Print
    .reply("Carol".to_string());  // answers Console::ReadLine

let result = greet_user().run_with(&mut script);
assert_eq!(script.remaining(), 0);
```

An operation arriving after the script is exhausted panics with the script's
name, the number of replies served, and the unexpected operation.

### Calling effectful functions from effectful functions

`call!` runs a sub-computation and transparently forwards its effects to the
caller's handler:

```rust
#[effectful]
fn main_task() -> i32 {
    let _: () = perform!(Logger::Info("starting".to_string()));
    let n: i32 = call!(subtask());  // subtask's effects reach the same handler
    n * 10
}
```

Both `perform!` and `call!` are only usable inside `#[effectful]` functions;
using them elsewhere is a single clear compile error.

## Multiple effect families

One `effect!` block can declare several families; they all land in the same
root enum, so one handler (or one chain) can serve them all:

```rust
effect! {
    File::Read (String) -> Result<String, std::io::Error>;
    File::Write ((String, String)) -> Result<(), std::io::Error>;

    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
}
// Generates: enum File, enum Logger, and enum Op { File(File), Logger(Logger) }
```

### Custom root enums

Two `effect!` blocks in the same module would both generate `Op` and collide —
this is a compile error, detected by a generated sentry type (covered by
[`tests/ui/duplicate_root.rs`](algae/tests/ui/duplicate_root.rs)). To declare
several blocks in one scope, name the roots:

```rust
effect! {
    root ConsoleOp;
    Console::Print (String) -> ();
    Console::ReadLine -> String;
}

effect! {
    root FileOp;
    File::Read (String) -> Result<String, String>;
}
```

`#[effectful(root = ConsoleOp)]` selects the root for a function (paths like
`root = my_module::ConsoleOp` work too). Separate roots can be merged into one
enum for a unified handler — `combine_roots!` takes the (imported) root enum
identifiers:

```rust
use my_effects::{ConsoleOp, FileOp};

algae::combine_roots!(pub AppOp = ConsoleOp, FileOp);
// Generates: enum AppOp { ConsoleOp(ConsoleOp), FileOp(FileOp) } + From impls
```

Alternatively, put each `effect!` block in its own module and keep the default
`Op` name per module. See
[`examples/multiple_effects_demo.rs`](algae/examples/multiple_effects_demo.rs)
and [`examples/custom_root_effects.rs`](algae/examples/custom_root_effects.rs).

## Error messages for type mismatches

When a handler replies with the wrong type, the panic (or `ReplyError` from
`try_take`) names both the expected and the actual type. Common std types are
recognized out of the box; register your own once at startup to get real names
instead of a bare `TypeId`:

```rust
algae::register_type::<MyDomainType>();
```

[`examples/test_error_messages.rs`](algae/examples/test_error_messages.rs)
demonstrates the messages.

## Relation to the theory

Algae follows the algebraic-effects-and-handlers model of Plotkin and Pretnar,
restricted to one-shot continuations:

| Theory | In algae |
|---|---|
| Effect signature | `effect!` declaration |
| Operation call | `perform!(Family::Op(...))` |
| Handler | `Handler<Op>` / `PartialHandler<Op>` |
| Computation | `Effectful<R, Op>` |
| Handling | `.handle(h).run()` / `run_checked` |

Because continuations are never captured, operations that require resuming a
continuation more than once (non-deterministic choice, backtracking search,
generators) cannot be expressed. Effectful functions compose with
`call!(sub())`, which forwards the sub-computation's effects to the caller's
handler; `Effectful::bind` sequences whole computations from outside, and
`Effectful::resume` exposes single steps for custom drivers.

[`tests/algebraic_laws.rs`](algae/tests/algebraic_laws.rs) verifies identity,
associativity, homomorphism, and commutativity properties of the implementation,
with extensive explanatory comments; [`examples/theory.rs`](algae/examples/theory.rs)
maps the concepts to running code.

## Performance characteristics

There are no benchmarks yet, so no numbers — but the costs are easy to
enumerate: one heap allocation per computation (the pinned coroutine), one
boxed allocation plus a runtime type check per performed effect, and the
coroutine suspend/resume itself. Effects in a hot loop will cost more than
direct calls; batch operations where that matters.

## Examples

Run any of these with `cargo run --example <name>` (add
`--no-default-features` for `no_macros`):

| Example | Shows |
|---|---|
| `readme` | The quick-start program with real and mock handlers |
| `typed_api` | Generated typed constructors and handler traits |
| `overview` | Guided tour of the examples, tests, and docs |
| `minimal` | Raw coroutine mechanics beneath `Effectful` |
| `effect_test` | Smallest end-to-end effect round trip |
| `debug` | Tiny scratchpad exercising the macros |
| `pure` | State management via effects |
| `console` | Interactive I/O with real and mock implementations |
| `explicit_vs_convenient` | `#[effectful]` sugar vs. hand-written coroutines |
| `no_macros` | Using the runtime with the macros feature disabled |
| `advanced` | Multi-family app: files, database, logging, tests |
| `multiple_effects_demo` | Single declaration vs. module separation trade-offs |
| `custom_root_effects` | `root` names and `combine_roots!` |
| `partial_handlers` | Panic-free composition with `run_checked` |
| `chained_handlers` | `.handle().handle()` chaining |
| `clean_chaining` | `begin_chain` in its simplest form |
| `variable_handler_chain` | Building chains of varying length |
| `vec_handler_flattening_demo` | `merge` vs. nested `VecHandler`s |
| `test_error_messages` | Type-mismatch diagnostics and `register_type` |
| `theory` | Theory-to-implementation mapping |

Regression suites live in [`algae/tests/`](algae/tests/), including the
algebraic-laws tests and a trybuild compile-fail suite.

## Development

The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml);
`rustup` picks it up automatically. The Makefile drives everything CI runs:

```bash
make ci-local   # full pipeline: fmt, clippy (strict), tests, examples, docs
make test       # unit + integration + doctests + macro tests
make dev        # quick loop: fmt, check, unit tests
make help       # everything else
```

Contributions are welcome — run `make ci-local` before opening a PR.

## License

MIT — see [LICENSE](LICENSE).

## Further reading

- [An Introduction to Algebraic Effects and Handlers](https://www.eff-lang.org/handlers-tutorial.pdf) — Pretnar's tutorial
- [Eff](https://www.eff-lang.org/) — the original algebraic-effects language
- [Koka](https://koka-lang.github.io/) — a language with typed effect handlers
- [OCaml effect handlers](https://ocaml.org/manual/effects.html) — one-shot effects in a mainstream language
- [Algebraic Effects for the Rest of Us](https://overreacted.io/algebraic-effects-for-the-rest-of-us/) — an accessible introduction
