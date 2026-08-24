# Algae Ergonomics Investigation

- **Date:** 2026-08-24, at commit `2772de0`
- **Method:** friction inventory drawn from algae's own source, tests, and
  examples; a prior-art survey of effect-system APIs (effing-mad, reffect,
  corophage, OCaml 5, Koka, plus mocking frameworks); and **compile-verified
  prototypes** of the main proposals against the real crate (a scratch crate
  with a path dependency on `algae`; every claim marked *verified* below was
  compiled and run, including a negative test proving the claimed compile
  error fires).

---

## 1. Where the friction actually is

Measured in this repository's own code (library, tests, examples):

| # | Friction | Evidence |
|---|---|---|
| F1 | **Annotation tax at perform sites.** Every effect call needs a type ascription; unit-returning effects need the `let _: () = perform!(...)` incantation. | **157** occurrences of `let _: () = perform!` and **32** typed `let x: T = perform!` bindings in this repo alone. |
| F2 | **Handlers are untyped.** `handle` returns `Box<dyn Any + Send>`; nothing checks the reply type against the `effect!` declaration until a runtime panic in `Reply::take`. The declared `-> Ret` in `effect!` is documentation only. | **297** `Box::new(...)` sites; the entire error-message registry (`register_type`, `TYPE_NAMES`) exists solely to make the resulting runtime panics readable. |
| F3 | **Handler match boilerplate.** Double-nested patterns (`Op::Console(Console::Print(msg))`) plus a wildcard arm that panics for families the handler doesn't own. | **31** `_ => panic!` arms in handlers across the repo. |
| F4 | **Mocks can't be inspected after the run.** Handlers are moved into `run_with`/`run`, so a test that records into its mock loses the recording. | Three tests carry the comment "handler is moved into the effectful computation, so we can't inspect its logs afterwards"; the promoted integration tests had to thread `Arc<Mutex<Vec<String>>>` through handlers to assert anything. |
| F5 | **Effectful functions don't compose.** There is no way to call one `#[effectful]` fn from another and forward its effects; `Effectful.gen` is private and there is no resume API. `bind` sequences only whole computations, CPS-style, from outside. | The one "nested effects" test (`nested_effects` in lib.rs) resorts to running the inner computation with its own separate handler mid-function. |
| F6 | **Per-test handler ceremony.** Every test defines a struct + `Handler` impl with a match, even when the answers are just "reply 42, then reply ()". | Every test module in lib.rs re-declares mock handler structs. |
| F7 | **Declaration-side rigidity.** Payloads must derive `Debug + Clone + PartialEq`; multi-parameter operations are double-parenthesized tuples (`Math::Add ((i32, i32))`) with unnamed fields; generated enums are always `pub`. | Documented in `effect!` since the audit; inherent to the macro. |
| F8 | **Misuse diagnostics.** `perform!` outside `#[effectful]` produces a wall of coroutine trait errors rather than one sentence. | Reproducible with any stray `perform!`. |
| F9 | **No effect polymorphism.** A function that uses only Console effects still has type `Effectful<T, Op>` over the whole program's `Op`. | Inherent to the single-root design; custom roots are the manual escape hatch. |

F1 + F2 are two faces of one defect: the type information the user already
wrote in `effect!` is thrown away, then demanded back at every perform site
and silently trusted at every handler site.

## 2. What prior art says

Full survey with sources in §6. The load-bearing findings:

**Convergent evolution on typed operations.** The three Rust effect libraries
built on coroutines/generators — effing-mad (2023, nightly), reffect (2025,
nightly), corophage (2026, stable, actively maintained) — independently
converged on the same core design: **one *type* per operation carrying its
reply type as an associated type** (`trait Effect { type Resume; }`), with a
generic perform whose return type is `E::Resume`, inferred from the argument
value. Result: zero caller annotations, compile-checked replies, no `dyn Any`.
Corophage's variant is the cleanest — perform is a method generic over `E`,
so no `PhantomData` marker plumbing is needed. Algae's only blocker is that
Rust enum variants are not types; the fix is macro-generated witness types
(§3, P1).

**One-shot with a single dynamic effect set is a defensible design.** OCaml 5
ships exactly algae's trade — one-shot continuations, no static effect
tracking, unhandled effects fail dynamically — and its manual defends both
choices (cheap implementation, linear-resource safety, retrofit
compatibility). The complaint OCaml users actually have is annotation
boilerplate, i.e. algae's F1/F2, not the missing effect rows.

**Row-polymorphic effect sets are a documented dead end in Rust.** effing-mad
implemented them via frunk coproducts and its own doc comments concede the
result: `handle` carries 12 type parameters, `transform` carries 21, and
"calling this function almost always requires annotating … 21 underscores."
Its two successors both declined to follow — reffect replaced coproducts with
a flat tagged union specifically to keep inference tractable, and corophage
simply doesn't offer effect-set transformation. Koka's rows are ergonomic
only because Koka has whole-program effect *inference*, which Rust does not.

**Handlers-by-`&mut` is a solved problem.** reffect provides
`impl Handler for &mut H` (mock inspectable after the run); corophage threads
`&mut S` state into handlers via `run_sync_stateful`. Both beat Drop-based
verification (mockall's approach has a documented hole when the mock never
drops).

**Composition syntax.** For calling effectful fns from effectful fns:
effing-mad uses a fake-field `.do_`, reffect uses `.await`, corophage uses
`invoke!(...)`. All are the same proc-macro rewrite: a resume loop that
re-yields the callee's effects.

**Test-handler ergonomics.** mockito's model — every scripted reply is
consume-once, exhausted entries fall through, so *declaration order is the
script* — needs zero extra syntax. mockall implements the same rule
internally (`!is_done()` fall-through) plus a diagnostics trick worth
copying: when only one candidate expectation exists, reuse it even when
exhausted so the failure reads "called twice, expected once" instead of
"nothing matched".

## 3. Proposals, ranked

Prototype code for everything marked *verified* is in the session scratchpad
(`ergo-proto/src/main.rs`), compiled and run against this crate.

### P1 — Operation witness types: inferred, compile-checked `perform!` *(verified)*

**The change.** `effect!` additionally generates, per operation, a witness
carrying the declared return type, plus a snake_case constructor; `perform!`
consumes any such witness and returns its reply type. Two equivalent shapes:

- *Prototyped shape:* constructors return `TypedOp<Op, Ret>` (a root op plus
  `PhantomData<Ret>`); `perform!` splits it into the raw op (yielded) and a
  phantom-typed extractor (applied after resume). The split is a real design
  requirement discovered while prototyping: extracting a field of the wrapper
  across the `yield` point is a partial move and does not compile.
- *Prior-art shape:* one unit/tuple struct per operation implementing
  `trait Operation { type Reply; fn into_op(self) -> Root; }`, with
  `perform!` using the effing-mad "mark" trick
  (`fn extractor<O: Operation>(_: &O) -> Extractor<O::Reply>`) to recover the
  reply type from the value. Same semantics; the struct-per-op shape also
  gives handlers something typed to match in a later phase.

Caller side, before and after (both compiled in the prototype):

```rust
// today
let _: () = perform!(Console::Print("What's your name?".to_string()));
let name: String = perform!(Console::ReadLine);
let n: i32 = perform!(Math::Add((1, 2)));

// proposed
perform!(console::print("What's your name?"));   // no let _: () =
let name = perform!(console::read_line());        // inferred String
let n = perform!(math::add(1, 2));                // inferred i32; tuple payload
                                                  // becomes named parameters
```

**Verified safety upgrade:** requesting the wrong type at a perform site is
now a *compile* error at that site (`error[E0308]: expected String, found
i32`) instead of a runtime panic inside the handler-reply plumbing. This
retires most of F1 and the caller half of F2 — and with it, most of the
reason `register_type`/the type-name registry exist.

**Migration.** Fully incremental: the enums, `Handler`, and `Box<dyn Any>`
internals stay; the old `perform!(Family::Variant(x))` form can keep working
(the macro can accept both, or ship as `perform!` v2 behind an edition-style
switch). Constructors flatten tuple payloads into named parameters
(`math::add(a, b)`), fixing the double-paren wart (part of F7) for free, and
can take `impl Into<String>` for string payloads.

**Costs/caveats.** Two namespaces per family (enum + constructor module) —
name them predictably (`snake_case(family)`); divergent/never-returning ops
need a `Reply = Infallible`-style story; constructor generation is the
largest single addition to the proc macro but is mechanical.

### P2 — Per-family typed handler traits *(verified)*

**The change.** `effect!` generates one trait per family with a typed method
per operation, plus an adapter struct implementing `PartialHandler` by
dispatching and boxing internally:

```rust
// generated
pub trait ConsoleOps {
    fn print(&mut self, msg: &str);
    fn read_line(&mut self) -> String;
}
pub struct HandleConsole<T>(pub T);   // impl PartialHandler<Op> generated

// user side — a mock console today vs. proposed
impl ConsoleOps for MockConsole {
    fn print(&mut self, msg: &str) { self.printed.push(msg.into()); }
    fn read_line(&mut self) -> String { self.input.clone() }
}
```

Composed through the existing chain
(`.begin_chain().handle(HandleConsole(mock)).handle(HandleMath(RealMath))` —
verified end-to-end). This retires the handler half of F2 and all of F3: no
boxing, no nested matches, no wildcard panic arms, and a handler that fails
to cover an operation is a *missing trait method* compile error. Precedent:
reffect's `#[group]` / `#[group_handler]` traits. The raw `Handler` trait
stays for whole-program handlers and back-compat.

### P3 — Handlers by `&mut` + closure handlers *(specified; needs in-crate impls)*

**The change.** Add blanket impls inside algae:

```rust
impl<Op, H: Handler<Op>> Handler<Op> for &mut H { ... }
impl<Op, H: PartialHandler<Op>> PartialHandler<Op> for &mut H { ... }
```

so tests run `computation.run_with(&mut mock)` and inspect the mock
afterwards — retiring F4 outright (reffect's README demonstrates exactly this
workflow). Cannot be prototyped externally (orphan rule), but it is the same
pattern std uses for `FnMut`, and there is no overlap with the existing
concrete impls (`VecHandler`, `HandlerWrapper`, `Box<dyn PartialHandler>`).

For inline handlers without a struct, the prototype verified a wrapper-based
adapter (`handler_fn(|op| ...)`), which composes in a chain today.
**Coherence warning from prior art:** if algae instead wants a *blanket*
`impl PartialHandler for F: FnMut(...)`, it genuinely overlaps with the
`&mut H` blanket (std makes `&mut F` an `FnMut` too). reffect solves this
with a defaulted `Marker` type parameter on the trait — which must be
designed in **from the start**, since retrofitting it breaks every bound.
Recommendation: ship the `&mut` blankets plus the `handler_fn` wrapper (no
marker needed, zero conflicts) and skip the closure blanket.

### P4 — `call!` delegation between effectful functions *(specified)*

**The change.** One new public method on `Effectful` (expose a resume step;
today `gen` is private, which is the sole blocker) plus a macro:

```rust
#[effectful]
fn main_program() -> String {
    perform!(logger::info("starting"));
    let name = call!(greet_user());      // forwards greet_user's effects
    format!("done: {name}")
}
```

expanding to a loop that resumes the sub-computation, re-yields each of its
effects outward, and feeds replies back — the same rewrite as reffect's
`.await` and corophage's `invoke!`. Retires F5; `bind` remains for external
sequencing. Since algae has a single root `Op`, no subset checking is needed
(callee and caller share the root; custom-root callees compose via the
existing `From` conversions). Syntax note: prior art favors `.await`
(familiar suspension syntax) — worth considering, but `call!` requires no
rewriting of arbitrary expressions inside the attribute macro, so it is the
cheaper first step.

### P5 — Scripted mock handlers *(verified)*

**The change.** A built-in test handler where declaration order is the
script; the prototype's minimal version already works:

```rust
let result = greet_user().run_with(ScriptHandler::new(vec![
    Box::new(()),                    // reply to Print
    Box::new("Carol".to_string()),   // reply to ReadLine
]));
```

Ship it with the diagnostics conventions from the mocking-framework survey:
consume-once entries with fall-through, open-endedness allowed only on the
last entry, the "single remaining candidate" trick so exhaustion reads
"called twice, expected once", and named scripts so panics identify the
script rather than an index. With P2, a *typed* variant becomes possible
(`Script::console().read_line_returns("Carol")`) — generated per family, no
`Box::new` in tests at all. Retires F6.

### P6 — Diagnostics package *(specified; cheapest of all)*

- A top-level `perform!` (and future `call!`) fallback whose body is just
  `compile_error!("perform! can only be used inside an #[effectful] function")`,
  shadowed by the real macro inside the attribute — corophage does exactly
  this, and it converts today's coroutine-trait error wall (F8) into one
  sentence. ~10 lines.
- `#[diagnostic::on_unimplemented]` on `Handler`/`PartialHandler` (and
  `Operation`, if P1 lands) explaining what to implement.
- Rename-in-place polish that was deferred in the audit: distinct name for
  the chain-extension `Handled::handle` when P1's breaking wave happens
  anyway.

### Explicitly rejected

- **Coproduct-based effect rows (fix for F9):** implemented once in Rust,
  documented by its own author as requiring "21 underscores" of annotation;
  abandoned by both successor libraries. Algae's single-root design matches
  what OCaml 5 shipped deliberately. If effect-set composition pressure grows,
  the ergonomic ceiling is corophage-style *alias-level* composition
  (`Effects![Cancel, ...IoEffects]`-style spread over custom roots), not row
  polymorphism in signatures.
- **Multi-shot effects:** contradicts the crate's founding design choice;
  effing-mad's opt-in version demands `Clone` coroutines on an unstable
  feature and infects locals with `Clone` bounds.
- **Passing owned ops to handlers** (would avoid payload clones): conflicts
  with `run_checked` returning the op on decline and with `VecHandler`
  fall-through; the clone tax is real but small, and `&op` matching is what
  every prior-art handler does too. Revisit only with representative
  benchmarks.

## 4. Suggested roadmap

**Wave 1 — additive, no breakage:** P6 diagnostics; P3 `&mut` blankets +
`handler_fn`; P5 `ScriptHandler`; P4 resume method + `call!`. Each is small,
independent, and immediately usable.

**Wave 2 — the typed API (0.2):** P1 witness constructors + inferred
`perform!`, and P2 typed handler traits, generated by `effect!` alongside the
existing enums. Old forms keep compiling during a deprecation window; the
type-name registry (`register_type`) becomes legacy once callers are typed.

**Wave 3 — declaration polish:** visibility/derive configuration in `effect!`
(the audit's deferred M6), named payload fields via the constructors, and a
decision on `Handled::handle` naming — natural to bundle with the 0.2
breaking wave.

## 5. Prototype artifacts

Scratch crate (session scratchpad, `ergo-proto/`): hand-written expansions of
P1 (TypedOp + `performt!`), P2 (ConsoleOps/MathOps + adapters), P3's
`handler_fn`, and P5's `ScriptHandler`, all running green against
`algae = { path = ... }` on the pinned nightly; plus the negative test
demonstrating P1's compile-time rejection (`E0308: expected String, found
i32` at the perform site).

## 6. Sources

- effing-mad — <https://github.com/rosefromthedead/effing-mad>,
  <https://docs.rs/effing-mad> (typed injections, `mark`/`Tagged` technique,
  the coproduct-rows cautionary tale, opt-in multi-shot)
- reffect — <https://github.com/js2xxx/reffect> (`.await` composition,
  `handler!` with match guards, `Handler for &mut H`, `Marker` coherence
  trick, flat tagged-union effect sets, `#[group]` handler traits)
- corophage — <https://corophage.rs/>, <https://github.com/romac/corophage>
  (stable-Rust, generic-method `yield_` needing no marker, `invoke!`,
  `run_sync_stateful(&mut state)`, `Effects![...]` spread,
  `compile_error!` fallback macros, explicit one-shot rationale)
- effers — <https://github.com/annieversary/effers> (traits-as-effects DI
  style; proc-macro visibility constraint)
- OCaml 5 effects — <https://ocaml.org/manual/5.4/effects.html>,
  <https://ocaml.org/manual/5.4/api/Effect.html>, PLDI'21 paper
  <https://arxiv.org/pdf/2104.00250> (GADT-typed `perform`, one-shot
  rationale, no-static-effect-tracking rationale, `Effect.Unhandled` carrying
  the op)
- Koka — <https://koka-lang.github.io/koka/doc/book.html> (`with` sugar,
  `val`/`fun`/`ctl` operation kinds, inferred effect rows, named handlers)
- mockall — <https://docs.rs/mockall> (expectation fall-through, exhaustion
  diagnostics, Drop-verification hole #396); mockito — <https://docs.rs/mockito>
  (declaration-order-as-script); faux — <https://docs.rs/faux> (`.once()`
  unlocking `FnOnce`); wiremock — <https://docs.rs/wiremock> (named mocks)
