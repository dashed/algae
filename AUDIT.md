# Algae Repository Audit

> **Remediation status (2026-08-24, same day):** All high-priority findings (H1–H4) and
> medium findings M1–M5, M7 (partial), and M8 have been fixed, plus low items L1, L4, L5,
> and L6. Verified end-to-end: `make ci-local` passes from a clean build on the now-pinned
> `nightly-2026-08-24`; 141 tests pass (56 lib + 14 algebraic-laws + 15 promoted
> regression tests + 1 trybuild compile-fail + 21 macro + 34 compiled doctests).
> Corrections to the original audit: (a) `minimal.rs` does name the `Coroutine` trait, so
> it legitimately keeps `coroutine_trait`; (b) whether the unused-feature warning was
> actively failing CI proved cache-dependent — the cleanup landed either way.
> **Deliberately not done:** M6 configurable derives/visibility in `effect!` (documented
> instead — an API design decision), L2 internal simplifications (the stored `TypeId` was
> a deliberate optimization, see commit `5b5c299`), L7 `Handled::handle` rename (API
> break), and a `try_perform!` checked variant (new feature, not a fix). The findings
> below are preserved as originally written.

- **Date:** 2026-08-24
- **Commit:** `a366323` (master, clean tree)
- **Toolchain used:** `rustc 1.100.0-nightly (fb6531d55 2026-08-23)` (resolved from the repo's floating `nightly` channel)
- **Scope:** full read of `algae/src/lib.rs` (3,750 lines), `algae-macros/src/lib.rs` (900 lines), tests, examples, Makefile, CI workflow, manifests; plus local build/test/clippy verification.

---

## 1. Overview

Algae is a two-crate nightly-Rust workspace implementing **one-shot (linear) algebraic effects** on top of native coroutines:

- **`algae`** — runtime: `Effect<Op>` (op + type-erased reply slot), `Reply` (one-shot typed extraction with a global `TypeId → name` registry for error messages), `Effectful<R, Op>` (pinned boxed coroutine), `Handler`/`PartialHandler` traits, `VecHandler` chaining, `bind` for monadic sequencing, and `combine_roots!`.
- **`algae-macros`** — proc macros: `effect!` (generates family enums + root enum + `From` impls + a sentry enum for duplicate-root detection), `#[effectful]` (rewrites a fn into a coroutine returning `Effectful<T, Root>`), and `perform!` (yield + typed take).

The README honestly labels the project as an experimental toy. The core design is sound and pleasant: no `unsafe` anywhere, one-shot semantics enforced by ownership (`Reply::take(self)`, `Effect::get_reply(self)`), and only 4 external dependencies (`syn`, `quote`, `proc-macro2`, `unicode-ident`).

### Verification results

| Check | Result |
|---|---|
| `cargo build --workspace` | ✅ passes |
| `cargo test --workspace` | ✅ 80/80 pass (54 lib + 14 algebraic-laws + 12 macro parser) |
| `cargo build -p algae --no-default-features` | ✅ passes (lib only) |
| `cargo check -p algae --no-default-features --examples` | ❌ fails (see M2) |
| `cargo clippy --lib --all-features -- -D warnings` | ✅ passes |
| `make clippy-strict` (part of `make ci-local`, which CI runs) | ❌ **fails on current nightly** (see H1) |
| Doctests | ⚠️ 48 doctests, **all `ignore`d — zero are ever compiled** (see H3) |

---

## 2. High-priority findings (things that are broken today)

### H1. CI is red on the current nightly: `unused_features` warning promoted to error

Most examples and the `algebraic_laws` test declare:

```rust
#![feature(coroutines, coroutine_trait, yield_expr)]
```

but never use the `Coroutine` trait directly, and the current nightly now emits `warning: feature 'coroutine_trait' is declared but not used` (23 occurrences across targets). `make clippy-strict` runs every example with `-D warnings`, so it errors out — verified locally:

```
error: feature `coroutine_trait` is declared but not used
 --> algae/examples/theory.rs:1:24
```

Since `.github/workflows/*` runs `make ci-local` → `clippy-strict`, **the GitHub CI pipeline will fail on its next run**. Fix: drop `coroutine_trait` from files that don't name the trait (only code that references `std::ops::Coroutine`/`CoroutineState` directly — `lib.rs` and the manual-coroutine examples — needs it).

### H2. `make test-error-detection` passes vacuously — the safety net tests nothing

The Makefile step asserts that `cargo check --example duplicate_root_test` fails, treating that as proof the sentry-enum duplicate-root detection works. But the file is named `duplicate_root_test.rs.disabled`, so **no such example target exists**; cargo fails with "no example target named `duplicate_root_test`", and the Makefile counts that as success. The check would keep passing even if the sentry mechanism were deleted. A `tests/ui`-style compile-fail test (e.g. with `trybuild`) is the right tool here.

### H3. Every doctest is `rust,ignore` — the extensive docs are never compile-checked

There are ~48 doc examples across both crates and all are marked `ignore` (verified in test output: "0 passed; 48 ignored"). For a library whose main asset is its documentation, nothing prevents doc rot — and it has already happened (see M8: docs promise `Default` impls the macro doesn't generate). Nightly doctests *can* run: put `#![feature(...)]` inside the fenced block and mark blocks `no_run` if execution is undesirable. Even converting a third of these to compiled doctests would materially protect the docs.

### H4. `UnhandledOpError::from` is a stub that returns garbage

`algae/src/lib.rs:1435-1444`:

```rust
impl<Op: std::fmt::Debug> From<UnhandledOp<Op>> for UnhandledOpError {
    fn from(unhandled: UnhandledOp<Op>) -> Self {
        let _debug_str = format!("{:?}", unhandled.0);   // computed, then discarded
        UnhandledOpError { op_name: "UnknownOp" }        // always this literal
    }
}
```

The doc on `op_name` says "The name of the unhandled operation type", but every conversion yields `"UnknownOp"` — and it allocates a debug string just to throw it away. Either populate a real name (requires `op_name: String` and `format!("{:?}", …)`) or delete the type; as shipped it silently destroys error information. Nothing in the repo exercises this path, which is why tests don't catch it.

---

## 3. Medium-priority findings (design, robustness, DX)

### M1. Floating `nightly` channel is the project's biggest reliability risk

`rust-toolchain.toml` pins only `channel = "nightly"` and CI uses `dtolnay/rust-toolchain@nightly`. The crate depends on unstable coroutine syntax/semantics that have changed repeatedly (H1 is this risk materializing in miniature). Pin a dated snapshot (e.g. `channel = "nightly-2026-08-23"`) and bump deliberately; CI reads the same file, so one edit updates both.

### M2. Examples missing `required-features = ["macros"]` break feature-gated builds

`algae/Cargo.toml` declares `required-features` for 13 examples, but these auto-discovered examples use the macros without declaring the requirement: `chained_handlers`, `clean_chaining`, `partial_handlers`, `test_custom_root_effectful`, `test_effectful_backwards_compatibility`, `test_effectful_scoping_fix`, `test_effectful_scoping_simple`, `test_manual_default`, `test_non_default_payload`, `test_send_across_threads`, `variable_handler_chain`, `vec_handler_flattening_demo`. Verified: `cargo check -p algae --no-default-features --examples` fails with "cannot find macro `effect`". Either add the stanzas or replace the whole hand-maintained list with `required-features` on each and keep `no_macros` as the only exception. (The long per-example list in both Cargo.toml and the Makefile is itself drift-prone — the Makefile's `examples` target already omits several examples that exist.)

### M3. `perform!` panics the proc macro on malformed input

`algae-macros/src/lib.rs:725`: `let input: syn::Expr = syn::parse(ts).unwrap();` — a syntax error inside `perform!(...)` aborts the macro with the opaque "proc macro panicked" diagnostic instead of a spanned error. Use `parse_macro_input!` (returns a proper `compile_error!`).

### M4. `#[effectful]` argument parsing is string-based and rejects paths

`algae-macros/src/lib.rs:529-548` parses the attribute by `args.to_string().replace(" ", "")` + `strip_prefix("root=")` + `parse_str::<Ident>`. Consequences:

- `#[effectful(root = other_module::MyOp)]` is impossible (only bare identifiers parse), which undercuts the multi-module custom-root story that `combine_roots!` exists for.
- Token-stream stringification is fragile (e.g. `root=r#type` or any future arg breaks oddly).

Parsing with syn (`syn::meta` or a small `Parse` impl accepting `root = syn::Path`) fixes both. Similarly, the macro does not reject `async fn` even though the docs say async is unsupported — it currently generates `async fn … -> Effectful<…>` (a future of an `Effectful`), a confusing artifact; an explicit spanned error would be kinder.

### M5. Overlapping, partially dead handler-composition API

The surface for "run a computation with handlers" is large for a ~1,700-line runtime: `run_with`, `run_checked`, `run_checked_with`, `handle`, `handle_all`, `begin_chain`, `Handled::handle` (a *different* `handle` for the `VecHandler` specialization), `handle_total`, `HandlerWrapper`, `IntoPartialHandler`, `IntoVecHandler`, `impl_into_vec_handler!`. Specific issues:

- **`IntoPartialHandler` is dead code**: defined, blanket-implemented, and exported from the prelude (`lib.rs:1447-1465`), but nothing consumes it.
- **Chaining ergonomics contradict the pitch**: `.begin_chain().handle(H1).handle(H2)` requires each `Hn: IntoVecHandler`, which has no blanket impl (it would conflict with the `VecHandler` impl), so users must hand-write an `IntoVecHandler` impl per handler or know about `impl_into_vec_handler!` — your own tests do this manually three times (`lib.rs:3323-3367`). Consider making `Handled::<VecHandler>::handle` take `H2: PartialHandler + Send + 'static` directly and keep a separate `merge`/`handle_vec` for flattening; that removes the trait and the macro.
- **Inconsistent unhandled-op behavior**: `VecHandler` also implements `Handler` by panicking (`lib.rs:1376-1390`), so `computation().handle(vec_handler).run()` panics where `.run_checked()` errors. That's a footgun worth documenting loudly or removing (forcing checked execution for `VecHandler`).
- `UnhandledOp` implements neither `Display` nor `std::error::Error`, so it doesn't compose with `?`/`anyhow` in user code.

### M6. `effect!` hard-codes visibility and derives

Generated family/root enums are always `pub` and always `#[derive(Debug, Clone, PartialEq)]` (`algae-macros/src/lib.rs:337-342, 364-367`). Any payload type lacking `Clone` or `PartialEq` makes the whole `effect!` block fail with derive errors pointing at generated code; non-`pub` effect definitions are impossible. Consider accepting a visibility token and making derives configurable (or at minimum documenting the payload requirements — the README/docs never state them). Note `combine_roots!` derives only `Debug`, so combined roots silently lose `Clone`/`PartialEq` that individual roots have — an inconsistency users will hit when writing tests that compare ops.

### M7. Panic-based type checking is the contract — but the panic paths could carry more context

By design, a handler returning the wrong type panics at `Reply::take` (good error message now, with the registry). Two softer spots:

- `perform!` expands to `__reply_opt.unwrap()` — a `None` resume (only possible from a hand-written driver) panics with a bare "unwrapped None" and no hint. `expect("effectful computation resumed without a Reply — did you call resume(None) mid-computation?")` costs nothing.
- There is no checked variant of `perform!` (e.g. `try_perform!` returning `Result<R, ReplyError>`) even though `try_take` exists; handler-side type bugs are always fatal to the process.

### M8. Documentation drift (already present)

- `effect!` docs claim it generates "`Default` implementations where applicable" (`algae-macros/src/lib.rs:216`); the macro generates none (the `test_manual_default` example exists precisely because of this).
- `lib.rs:86` claims "Zero-cost abstractions: minimal runtime overhead" — every `perform!` costs at least one heap allocation (the boxed reply) plus dynamic dispatch through `dyn Any`, and each `Effectful` is a pinned `Box`. "Low-cost" (as the README says) is accurate; "zero-cost" is not.
- README's git-dependency snippet still says `https://github.com/your-username/algae.git`.
- The macro-crate module comment (`algae-macros/src/lib.rs:375-380`) reads like a message to a past collaborator ("The rest of the file is identical to what you had") — leftover AI/pairing narration worth deleting.

---

## 4. Low-priority findings

- **L1. Registry details** (`lib.rs:298-320`): `register_type` silently no-ops if the mutex is poisoned — arguably fine, but a `let _ =`-style comment would make the intent explicit. Also, pre-registered names are short (`"String"`) while `register_type` stores `std::any::type_name` (fully qualified, e.g. `my_crate::foo::Bar`), so error messages mix formats. A nicer long-term design: have handlers build replies through a typed constructor (`Reply::of::<T>(value)`) that captures `type_name::<T>()` statically — that would delete the global registry entirely, at the cost of an API change to `Handler`.
- **L2. Redundant state**: `Stored.type_id` duplicates what `stored.value.as_ref().type_id()` already provides; and the driver loop round-trips the handler's reply through the `Effect.reply` slot (`handle` → `fill_boxed` → `get_reply`) when it could construct a `Reply` directly. Both are simplification opportunities, not bugs.
- **L3. Sentry enum** works (nice trick), but its error message is only helpful if the user reads the enum name; the disabled example documenting it can't be compiled by anyone (see H2).
- **L4. CI hygiene**: `actions/cache@v3` is deprecated (v4 is current); the cache key doesn't include the toolchain version, so a nightly bump reuses a stale `target/`.
- **L5. Publishing metadata missing**: neither crate has `license`, `description`, `repository`, `readme`, or `rust-version` fields; `cargo publish` would refuse. The MIT `LICENSE` file exists but the manifests don't reference it. Fine for a toy, blocking for the crates.io plan the README hints at.
- **L6. Example naming**: 9 of 25 examples are named `test_*`, blurring the line between examples and tests. The `test_effectful_scoping_*` / `backwards_compatibility` files are really regression tests and would serve better under `tests/` (where they'd also run in CI — right now `make ci-local` only *compiles* examples, so their `assert!`s never execute except for the 4 in `run-examples`, which CI doesn't run).
- **L7. `Handled` naming collision**: `Effectful::handle` and `Handled::<VecHandler>::handle` are different methods with different bounds; error messages when the wrong one resolves are confusing. A distinct name for the chain-extension method (`and_handle`/`then`) would help.

---

## 5. Security review

Nothing concerning:

- **No `unsafe` code** in either crate (verified by grep).
- **Supply chain**: only `syn`/`quote`/`proc-macro2`/`unicode-ident`, all pinned in `Cargo.lock` (v4). No build scripts beyond those standard crates.
- **No I/O, network, env, or process access** in the library; examples only use stdout/stdin.
- The global `TYPE_NAMES` registry is `OnceLock<Mutex<…>>` — thread-safe, no ordering hazards; worst case a lost registration under poisoning (L1).
- CI has no secrets usage; workflow permissions are default (could be narrowed to `contents: read`, a one-line hardening).
- One denial-of-service-shaped nit, inherent to design: `run_with`/`VecHandler as Handler` panic on unhandled ops — in a library consumer that's a crash vector if op coverage regresses; `run_checked` is the safe path and could be promoted as the default in docs.

## 6. Test assessment

Strong for a toy project: 80 passing tests, including genuinely valuable ones (one-shot invariant panics, handler ordering, algebraic-law properties, before/after registry behavior). Gaps:

1. **No compile-fail (UI) tests** — the two most interesting guarantees (duplicate-root sentry, macro misuse errors) are untested (H2). `trybuild` covers this cheaply.
2. **Doctests never run** (H3).
3. **`bind` is untested** — the monadic-bind implementation in `lib.rs:628-669` (the subtlest coroutine code in the repo, with the mid-stream `reply = None` reset) has no test or example exercising it at all. It reads correct to me, but it's exactly the code that should have an algebraic-laws test (left/right identity and associativity are tested via `#[effectful]` composition, not via `bind`).
4. **`UnhandledOpError`, `HandlerWrapper`/`handle_total`, `impl_into_vec_handler!`** have no coverage (and H4 shows what that allows).
5. Examples containing assertions aren't executed in CI (L6).

## 7. Prioritized recommendations

1. Fix the nightly breakage (H1): remove unused `coroutine_trait` feature declarations; then pin a dated nightly (M1).
2. Replace `make test-error-detection` with a `trybuild` compile-fail test and re-enable the duplicate-root case (H2).
3. Fix or delete `UnhandledOpError` (H4); add `Display`/`Error` to `UnhandledOp` (M5).
4. Add `required-features` to the 12 unannotated macro-using examples (M2).
5. Make a doctest-compilation pass: convert `ignore` → `no_run` where feasible (H3), and fix the drifted claims (M8).
6. DX pass on macros: `parse_macro_input!` in `perform!` (M3), syn-based attr parsing accepting `root = path::To::Op`, explicit `async fn` rejection (M4).
7. API diet: delete `IntoPartialHandler`, rework `IntoVecHandler` chaining so plain `PartialHandler`s chain without boilerplate impls, and decide whether `VecHandler as Handler` (panic on miss) should exist (M5).
8. When the crates.io ambition becomes real: manifest metadata (L5), configurable visibility/derives in `effect!` (M6), promote `test_*` examples into `tests/` (L6).
