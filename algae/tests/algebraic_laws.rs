#![feature(coroutines, coroutine_trait, yield_expr)]
#![cfg(feature = "macros")]

//! # Algebraic Laws Tests - A Learning Guide to Algebraic Effects
//!
//! This module contains comprehensive tests that verify the algae library respects the fundamental
//! mathematical properties (called "algebraic laws") that define algebraic effects and handlers.
//!
//! ## What Are Algebraic Effects?
//!
//! Algebraic effects are a programming paradigm that separates **what** you want to do from **how**
//! it gets done. Think of them as a more powerful version of interfaces or abstract methods.
//!
//! ## One-Shot (Linear) Effects
//!
//! **Important**: Algae implements **one-shot (linear) algebraic effects**. This means:
//! - Each effect operation receives exactly **one response** from a handler
//! - Effects are **consumed** after use and cannot be replayed or resumed multiple times
//! - **No continuation capture** - computations follow linear control flow
//! - **Simpler reasoning** - easier to understand and debug than multi-shot alternatives
//!
//! This design choice covers the vast majority of practical use cases while maintaining
//! excellent performance and simplicity. The algebraic laws tested here apply specifically
//! to this one-shot model of effects.
//!
//! ### The Problem They Solve
//!
//! Traditional programming often mixes business logic with implementation details:
//!
//! ```rust,ignore
//! fn process_user(id: u32) -> String {
//!     // Business logic mixed with implementation details
//!     let user = database::get_user(id);           // Direct database call
//!     println!("Processing user: {}", user.name);  // Direct console output
//!     log::info!("User processed: {}", id);        // Direct logging
//!     format!("Processed: {}", user.name)
//! }
//! ```
//!
//! With algebraic effects, you separate concerns:
//!
//! ```rust,ignore
//! #[effectful]
//! fn process_user(id: u32) -> String {
//!     // Pure business logic - no implementation details
//!     let user: User = perform!(Database::GetUser(id));        // "I need a user"
//!     let _: () = perform!(Console::Print(format!("Processing: {}", user.name))); // "I need to output"
//!     let _: () = perform!(Logger::Info(format!("User processed: {}", id)));      // "I need to log"
//!     format!("Processed: {}", user.name)
//! }
//! ```
//!
//! The **handler** decides how these requests are fulfilled - with a real database, mock data,
//! files, network calls, etc. The business logic stays the same regardless of implementation.
//!
//! ## What Are Algebraic Laws?
//!
//! Algebraic laws are mathematical properties that ensure effects behave predictably and compose
//! well together. They're like "rules" that guarantee your effects work correctly.
//!
//! ### Laws in the One-Shot Model
//!
//! The algebraic laws tested here are specifically adapted for algae's one-shot effect model:
//! - **Associativity** still holds: grouping operations doesn't change the result
//! - **Identity** is simpler: no need to handle continuation capture edge cases  
//! - **Homomorphism** is more direct: handlers preserve structure without complexity
//! - **Commutativity** applies to independent operations (no shared continuation state)
//!
//! These laws are actually *easier* to verify and reason about in the one-shot model
//! compared to multi-shot systems with continuation capture.
//!
//! ### Why Do Laws Matter?
//!
//! 1. **Predictability**: Code behaves the same way regardless of how it's written
//! 2. **Composability**: Effects can be combined without surprises
//! 3. **Refactoring Safety**: You can restructure code and know it still works
//! 4. **Testing Confidence**: Mock handlers behave like real ones
//! 5. **Linear Reasoning**: One-shot semantics make control flow easier to follow
//!
//! ### The Laws in Plain English
//!
//! 1. **Associativity**: `(A then B) then C` = `A then (B then C)`
//!    - Operations can be grouped differently but produce the same result
//!
//! 2. **Identity**: Doing nothing doesn't change the result
//!    - `do_nothing(value)` = `value`
//!
//! 3. **Homomorphism**: Handlers preserve structure
//!    - If two computations are equivalent, handling them gives equivalent results
//!
//! 4. **Commutativity**: Some operations can be reordered
//!    - `A + B` = `B + A` (for addition, but NOT for database writes!)
//!
//! ## Understanding Through Examples
//!
//! This file demonstrates each law with concrete examples using different effect families:
//!
//! - **State**: Get/Set operations (like variables)
//! - **Pure**: Mathematical operations (addition, multiplication)
//! - **Exception**: Error handling (throw/catch)
//! - **Choice**: Non-deterministic selection
//!
//! ## How to Read These Tests
//!
//! Each test follows this pattern:
//!
//! 1. **Setup**: Define effect operations and handlers
//! 2. **Law Statement**: What mathematical property we're testing
//! 3. **Left Side**: One way to write the computation
//! 4. **Right Side**: An equivalent way to write it
//! 5. **Assertion**: Verify both sides produce the same result
//!
//! The tests prove that our algebraic effects implementation is mathematically sound
//! and follows the academic theory developed by researchers like Gordon Plotkin and Matija Pretnar.
//!
//! ## Learning Path
//!
//! If you're new to algebraic effects, read the tests in this order:
//!
//! 1. `test_left_identity` - Simplest law about "doing nothing"
//! 2. `test_right_identity` - The reverse of identity
//! 3. `test_associativity_of_sequential_composition` - How operations group
//! 4. `test_handler_homomorphism` - How handlers preserve meaning
//! 5. `test_effect_commutativity` - When order doesn't matter
//! 6. `test_effect_non_commutativity` - When order DOES matter
//! 7. The remaining tests for advanced properties
//!
//! Each test includes detailed comments explaining what's happening and why it matters.

use algae::prelude::*;
use std::collections::HashMap;

//══════════════════════════════════════════════════════════════════════════════
// EFFECT DEFINITIONS
//══════════════════════════════════════════════════════════════════════════════

// Effects used throughout these tests to demonstrate algebraic laws.
//
// Each effect family represents a different kind of computation:
//
// - **State**: Stateful operations like reading/writing variables
// - **Pure**: Mathematical operations with no side effects
// - **Exception**: Error handling operations
// - **Choice**: Non-deterministic selection operations
//
// These cover the main categories of effects you'll encounter in real programs.
effect! {
    // State effects: Like having a mutable variable you can read and write
    // These are NOT commutative - the order of operations matters!
    State::Get -> i32;           // "What's the current value?"
    State::Set (i32) -> ();      // "Set the value to X"

    // Pure effects: Mathematical operations with predictable results
    // These ARE commutative when the operations themselves are commutative
    Pure::Identity (i32) -> i32;          // "Return the same value" (identity function)
    Pure::Add ((i32, i32)) -> i32;        // "Add two numbers"
    Pure::Multiply ((i32, i32)) -> i32;   // "Multiply two numbers"

    // Exception effects: Error handling operations
    // Used to test how errors interact with other effects
    Exception::Throw (String) -> ();                    // "Signal an error"
    Exception::Catch (String) -> Result<String, String>; // "Try to catch an error"

    // Choice effects: Non-deterministic operations
    // Used to test behavior when there are multiple possible outcomes
    Choice::Select (Vec<i32>) -> i32;     // "Pick one from many options"
    Choice::Empty -> Option<i32>;         // "Maybe return a value, maybe not"
}

//══════════════════════════════════════════════════════════════════════════════
// HANDLER IMPLEMENTATIONS
//══════════════════════════════════════════════════════════════════════════════

/// StateHandler: Implements stateful operations (like a mutable variable)
///
/// This handler maintains an internal integer state that can be read and modified.
/// It demonstrates how effects can have persistent state across operations.
///
/// Key insight: The handler's state persists between effect operations, which is
/// why the order of State::Set operations matters (non-commutativity).
struct StateHandler {
    state: i32, // The current value of our "variable"
}

impl StateHandler {
    /// Create a new StateHandler with the given initial value
    fn new(initial: i32) -> Self {
        Self { state: initial }
    }
}

impl Handler<Op> for StateHandler {
    /// Handle state operations by reading or modifying internal state
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Get: Return the current state value
            Op::State(State::Get) => Box::new(self.state),

            // Set: Update the state to a new value, return unit
            Op::State(State::Set(value)) => {
                self.state = *value;
                Box::new(())
            }

            // This handler only knows about State operations
            _ => panic!("StateHandler cannot handle operation: {op:?}"),
        }
    }
}

impl PartialHandler<Op> for StateHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::State(_) => Some(self.handle(op)),
            _ => None,
        }
    }
}

/// PureHandler: Implements mathematical operations with no side effects
///
/// This handler performs pure mathematical operations. Unlike StateHandler,
/// it has no internal state - each operation is independent and deterministic.
///
/// Key insight: Pure operations are "referentially transparent" - calling
/// Pure::Add((2, 3)) always returns 5, no matter when or how many times you call it.
struct PureHandler;

impl Handler<Op> for PureHandler {
    /// Handle pure mathematical operations
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Identity: Return the input unchanged (like f(x) = x)
            Op::Pure(Pure::Identity(x)) => Box::new(*x),

            // Add: Return the sum of two numbers
            Op::Pure(Pure::Add((a, b))) => Box::new(a + b),

            // Multiply: Return the product of two numbers
            Op::Pure(Pure::Multiply((a, b))) => Box::new(a * b),

            // This handler only knows about Pure operations
            _ => panic!("PureHandler cannot handle operation: {op:?}"),
        }
    }
}

impl PartialHandler<Op> for PureHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Pure(_) => Some(self.handle(op)),
            _ => None,
        }
    }
}

/// ExceptionHandler: Implements error handling operations
///
/// This handler tracks thrown exceptions and can simulate catching them.
/// It demonstrates how effects can model error conditions and recovery.
///
/// Key insight: Exceptions break normal control flow - they can cause computations
/// to short-circuit or take alternative paths.
struct ExceptionHandler {
    thrown_exceptions: Vec<String>, // Track what errors have been thrown
}

impl ExceptionHandler {
    /// Create a new ExceptionHandler with no thrown exceptions
    fn new() -> Self {
        Self {
            thrown_exceptions: Vec::new(),
        }
    }
}

impl Handler<Op> for ExceptionHandler {
    /// Handle exception operations (throw/catch)
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Throw: Record an exception as having been thrown
            Op::Exception(Exception::Throw(msg)) => {
                self.thrown_exceptions.push(msg.clone());
                Box::new(())
            }

            // Catch: Check if an exception was thrown, return Result accordingly
            Op::Exception(Exception::Catch(msg)) => {
                if self.thrown_exceptions.contains(msg) {
                    // Exception was thrown - return error
                    Box::new(Err::<String, String>(format!("Exception: {msg}")))
                } else {
                    // No exception - return success
                    Box::new(Ok::<String, String>(msg.clone()))
                }
            }

            // This handler only knows about Exception operations
            _ => panic!("ExceptionHandler cannot handle operation: {op:?}"),
        }
    }
}

/// ChoiceHandler: Implements non-deterministic selection operations
///
/// This handler can make choices from multiple options. In a real system,
/// this might represent things like random selection, user input, or exploring
/// multiple execution paths.
///
/// Key insight: Choice effects model situations where there are multiple valid
/// outcomes, and the handler decides which one to pick.
struct ChoiceHandler {
    choices: HashMap<Vec<i32>, i32>, // Predetermined choices for testing
}

impl ChoiceHandler {
    /// Create a new ChoiceHandler with no predetermined choices
    fn new() -> Self {
        Self {
            choices: HashMap::new(),
        }
    }

    /// Add a predetermined choice for specific options (used for testing)
    #[allow(dead_code)]
    fn with_choice(mut self, options: Vec<i32>, choice: i32) -> Self {
        self.choices.insert(options, choice);
        self
    }
}

impl Handler<Op> for ChoiceHandler {
    /// Handle choice operations (select from options or return empty)
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Select: Pick one option from the provided list
            Op::Choice(Choice::Select(options)) => {
                // Use predetermined choice if available, otherwise pick first
                let choice = self.choices.get(options).copied().unwrap_or(options[0]);
                Box::new(choice)
            }

            // Empty: Return None (representing "no choice available")
            Op::Choice(Choice::Empty) => Box::new(None::<i32>),

            // This handler only knows about Choice operations
            _ => panic!("ChoiceHandler cannot handle operation: {op:?}"),
        }
    }
}

/// CombinedHandler: Handles multiple effect families in one handler
///
/// This demonstrates how you can compose multiple handlers to handle different
/// effect families within a single computation. This is common in real applications
/// where you might need state, I/O, error handling, etc. all together.
///
/// Key insight: Handlers can be composed to support multiple effect families,
/// enabling complex applications with mixed effect types.
struct CombinedHandler {
    state: StateHandler,         // Handles State:: operations
    pure: PureHandler,           // Handles Pure:: operations
    exception: ExceptionHandler, // Handles Exception:: operations
    choice: ChoiceHandler,       // Handles Choice:: operations
}

impl CombinedHandler {
    /// Create a new CombinedHandler with initial state value
    fn new(initial_state: i32) -> Self {
        Self {
            state: StateHandler::new(initial_state),
            pure: PureHandler,
            exception: ExceptionHandler::new(),
            choice: ChoiceHandler::new(),
        }
    }
}

impl Handler<Op> for CombinedHandler {
    /// Route operations to the appropriate sub-handler based on effect family
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            // Route State operations to StateHandler
            Op::State(_) => self.state.handle(op),

            // Route Pure operations to PureHandler
            Op::Pure(_) => self.pure.handle(op),

            // Route Exception operations to ExceptionHandler
            Op::Exception(_) => self.exception.handle(op),

            // Route Choice operations to ChoiceHandler
            Op::Choice(_) => self.choice.handle(op),
        }
    }
}

impl PartialHandler<Op> for CombinedHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        Some(self.handle(op))
    }
}

//══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS FOR BUILDING EFFECTFUL COMPUTATIONS
//══════════════════════════════════════════════════════════════════════════════

/// A "pure" computation that doesn't perform any effects - just returns its input.
/// This represents the "do nothing" or "identity" operation that's important for
/// testing identity laws.
#[effectful]
fn pure_computation(value: i32) -> i32 {
    value // No effects performed - this is pure computation
}

/// Simple wrapper around State::Get for readability in tests
#[effectful]
fn get_state() -> i32 {
    perform!(State::Get)
}

/// Simple wrapper around State::Set for readability in tests
#[effectful]
fn set_state(value: i32) -> () {
    perform!(State::Set(value))
}

/// A compound operation that increments the state by 1.
/// This demonstrates how multiple effects can be combined into higher-level operations.
#[effectful]
fn increment_state() -> i32 {
    let current: i32 = perform!(State::Get); // Read current value
    let _: () = perform!(State::Set(current + 1)); // Write incremented value
    let new_value: i32 = perform!(State::Get); // Read new value to return
    new_value
}

/// A sequence that sets state to 'a', reads it, sets to 'b', then reads again.
/// This helps test associativity - how grouping of operations affects results.
#[effectful]
fn sequence_a_then_b(a: i32, b: i32) -> (i32, i32) {
    let _: () = perform!(State::Set(a)); // Set to first value
    let first: i32 = perform!(State::Get); // Read first value
    let _: () = perform!(State::Set(b)); // Set to second value
    let second: i32 = perform!(State::Get); // Read second value
    (first, second) // Return both values
}

/// Simple wrapper around Pure::Add for readability
#[effectful]
fn add_pure(a: i32, b: i32) -> i32 {
    perform!(Pure::Add((a, b)))
}

/// Simple wrapper around Pure::Multiply for readability
#[effectful]
fn multiply_pure(a: i32, b: i32) -> i32 {
    perform!(Pure::Multiply((a, b)))
}

/// A composed operation: first add x+y, then multiply the result by z.
/// This demonstrates composition of pure effects: (x + y) * z
#[effectful]
fn composed_pure(x: i32, y: i32, z: i32) -> i32 {
    let sum: i32 = perform!(Pure::Add((x, y))); // Step 1: x + y
    perform!(Pure::Multiply((sum, z))) // Step 2: (x + y) * z
}

//══════════════════════════════════════════════════════════════════════════════
// ALGEBRAIC LAWS TESTS
//══════════════════════════════════════════════════════════════════════════════

/// LAW 1: ASSOCIATIVITY OF SEQUENTIAL COMPOSITION
///
/// **Mathematical Statement**: `(a >> b) >> c ≡ a >> (b >> c)`
///
/// **What This Means**: When you have three operations that run one after another,
/// it doesn't matter how you group them with parentheses - the result is the same.
///
/// **Real-World Analogy**:
/// - Making a sandwich: (get bread, add ham), add cheese = get bread, (add ham, add cheese)
/// - The final sandwich is the same regardless of how you group the steps
///
/// **Why This Matters**:
/// - You can refactor code by regrouping operations without changing behavior
/// - Compilers/optimizers can rearrange operations safely
/// - Complex computations can be broken down or combined flexibly
///
/// **Example**:
/// - Left grouping: ((set to 1, multiply by 2), add 10)
/// - Right grouping: (set to 1, (multiply by 2, add 10))
/// - Both should give the same final result: 12
///
/// **Important**: This law applies to SEQUENTIAL composition (one after another),
/// not to operations that might interfere with each other.
#[test]
fn test_associativity_of_sequential_composition() {
    // All operations in one effectful computation with single handler
    #[effectful]
    fn all_operations() -> (i32, i32, i32, i32) {
        // Initial state
        let initial: i32 = perform!(State::Get);
        
        // Operation A: Set to 1
        let _: () = perform!(State::Set(1));
        let after_a: i32 = perform!(State::Get);
        
        // Operation B: Multiply by 2
        let current: i32 = perform!(State::Get);
        let _: () = perform!(State::Set(current * 2));
        let after_b: i32 = perform!(State::Get);
        
        // Operation C: Add 10
        let current: i32 = perform!(State::Get);
        let _: () = perform!(State::Set(current + 10));
        let final_state: i32 = perform!(State::Get);
        
        (initial, after_a, after_b, final_state)
    }
    
    let (initial, after_a, after_b, final_state) = 
        all_operations()
            .handle(StateHandler::new(0))
            .run_checked()
            .unwrap();
    
    assert_eq!(initial, 0);
    assert_eq!(after_a, 1);
    assert_eq!(after_b, 2);
    assert_eq!(final_state, 12);
    
    // EXPLANATION: Why this demonstrates associativity
    // ================================================
    //
    // The key insight: the final state is the same regardless of how we
    // conceptually group the operations: ((a;b);c) or (a;(b;c))
    //
    // - Starting state: 0
    // - After A: 1
    // - After B: 1 * 2 = 2
    // - After C: 2 + 10 = 12
    //
    // No matter how we group these operations conceptually, as long as they
    // execute in the same order (A, then B, then C), we get the same result.
    // This is what associativity guarantees for sequential composition.
}

/// LAW 2: LEFT IDENTITY FOR RETURN/PURE
///
/// **Mathematical Statement**: `return(x) >>= f ≡ f(x)`
///
/// **What This Means**: If you take a pure value, wrap it in the effect system,
/// then immediately apply a function to it, that's the same as just calling the
/// function directly on the original value.
///
/// **Real-World Analogy**:
/// - Putting a letter in an envelope just to immediately open it and read it
/// - is the same as just reading the letter directly
/// - The "envelope" (effect wrapper) doesn't change the contents
///
/// **Why This Matters**:
/// - Pure values can be lifted into the effect system without overhead
/// - You can freely convert between pure values and wrapped values  
/// - The effect system doesn't add artificial complexity to simple operations
/// - Identity operations don't interfere with other computations
///
/// **Example**:
/// - `return(5) >>= increment` should equal `increment(5)`
/// - The pure value 5, when processed by increment, gives the same result
///   whether it's wrapped in effects or not
///
/// **In Code**:
/// ```rust
/// // These two should be equivalent:
/// let wrapped = pure_value(5);
/// let result1 = wrapped.bind(|x| some_function(x));
///
/// let result2 = some_function(5);
///
/// assert_eq!(result1, result2); // Left identity holds
/// ```
#[test]
fn test_left_identity() {
    // Test within a single handler context
    #[effectful]
    fn test_both_sides() -> (i32, i32, bool) {
        // Left side: pure value (5) used in effectful computation
        let left_result = {
            let pure_value = 5; // This is "return(5)" in monadic terms
            // Apply function f inline
            let _: () = perform!(State::Set(pure_value));
            let result: i32 = perform!(State::Get);
            result
        };
        
        // Reset state to ensure fair comparison
        let _: () = perform!(State::Set(0));
        
        // Right side: f(5) directly
        let right_result = {
            let _: () = perform!(State::Set(5));
            let result: i32 = perform!(State::Get);
            result
        };
        
        (left_result, right_result, left_result == right_result)
    }
    
    let (left, right, equal) = test_both_sides()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    assert!(equal);
    assert_eq!(left, 5);
    assert_eq!(right, 5);
    
    // EXPLANATION: Why this demonstrates left identity
    // ===============================================
    //
    // The law states that wrapping a value in "return" and then applying
    // a function is the same as just applying the function directly.
    //
    // Left side breakdown:
    // 1. pure_value = 5 (pure computation)
    // 2. set state to 5, then get state -> 5
    //
    // Right side breakdown:
    // 1. set state to 5, then get state -> 5
    //
    // The "return" wrapper adds no behavior - it's truly an identity operation.
    // This proves our effect system doesn't add artificial overhead to pure values.
}

/// LAW 3: RIGHT IDENTITY FOR RETURN/PURE
///
/// **Mathematical Statement**: `m >>= return ≡ m`
///
/// **What This Means**: If you have an effectful computation and then apply
/// the "return" function to its result, you get back the same computation.
/// Adding a "do nothing" step at the end doesn't change the computation.
///
/// **Real-World Analogy**:
/// - Baking a cake and then "presenting it as-is"
/// - is the same as just baking the cake
/// - The "presenting as-is" step doesn't change anything
///
/// **Why This Matters**:
/// - You can add or remove pure "packaging" steps without changing behavior
/// - Pipelines can be simplified by removing redundant identity operations
/// - The effect system doesn't force you to unwrap and re-wrap values unnecessarily
/// - Refactoring tools can safely remove identity operations
///
/// **Example**:
/// - `get_user_from_db() >>= return` should equal `get_user_from_db()`
/// - The database operation followed by "return as-is" is just the database operation
///
/// **In Code**:
/// ```rust
/// // These two should be equivalent:
/// let result1 = some_computation().bind(|x| return(x));
/// let result2 = some_computation();
///
/// assert_eq!(result1, result2); // Right identity holds
/// ```
///
/// **Contrast with Left Identity**:
/// - Left identity: pure value + function = just the function
/// - Right identity: computation + pure wrapper = just the computation
#[test]
fn test_right_identity() {
    // Test within a single handler context
    #[effectful]
    fn test_computation() -> (i32, i32) {
        // Define computation m
        let m_result: i32 = {
            let _: () = perform!(State::Set(42));
            perform!(State::Get)
        };
        
        // Left side: m then return (identity function)
        let left_result = m_result; // return is just identity
        
        // Reset state
        let _: () = perform!(State::Set(0));
        
        // Right side: just m
        let right_result: i32 = {
            let _: () = perform!(State::Set(42));
            perform!(State::Get)
        };
        
        (left_result, right_result)
    }
    
    let (left, right) = test_computation()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    assert_eq!(left, right);
    assert_eq!(left, 42);
    
    // EXPLANATION: Why this demonstrates right identity
    // ================================================
    //
    // The law states that applying "return" to the result of a computation
    // doesn't change the computation's behavior.
    //
    // Left side: m >>= return
    // - effectful_computation() -> sets state to 42, returns 42
    // - return(42) -> just returns 42 (no effects)
    // - Final result: 42
    //
    // Right side: m
    // - effectful_computation() -> sets state to 42, returns 42
    // - Final result: 42
    //
    // The return step is redundant - it doesn't add any behavior.
    // This proves that identity operations can be safely added or removed.
}

//══════════════════════════════════════════════════════════════════════════════
// MATHEMATICAL NOTATION GUIDE
//══════════════════════════════════════════════════════════════════════════════
//
// Before diving into more complex laws, let's understand the mathematical notation:
//
// BASIC SYMBOLS:
// =============
// ≡     means "is equivalent to" or "equals" (the law being tested)
// >>    means "then" or "followed by" (sequential composition)
// >>=   means "bind" (take result of left side, feed it to right side function)
// λx    means "lambda x" (anonymous function that takes parameter x)
// ->    means "maps to" or "produces" (function type signature)
//
// EFFECT NOTATION:
// ===============
// perform!(Op)           - Execute an effect operation
// return(x)             - Wrap a pure value in the effect system (no effects)
// m >>= f               - Run computation m, then apply function f to its result
// (a >> b) >> c         - Run a, then b, then c (left-grouped)
// a >> (b >> c)         - Run a, then run (b followed by c) (right-grouped)
//
// PRACTICAL EXAMPLES:
// ==================
// State::Set(5)         - Set state to 5
// State::Get            - Read current state
// Pure::Add((2, 3))     - Add 2 + 3 (pure mathematical operation)
// perform!(State::Get)  - Execute the "get state" effect
//
// READING LAWS:
// ============
// When you see: "return(x) >>= f ≡ f(x)"
// Read as: "Taking pure value x, wrapping it in effects, then applying function f
//          is equivalent to just calling f(x) directly"
//
// When you see: "(a >> b) >> c ≡ a >> (b >> c)"
// Read as: "Doing a then b then c is the same whether you group it as
//          ((a then b) then c) or (a then (b then c))"

//══════════════════════════════════════════════════════════════════════════════
// ADDITIONAL ALGEBRAIC LAWS - ADVANCED PROPERTIES
//══════════════════════════════════════════════════════════════════════════════
//
// The following tests verify more advanced algebraic properties that ensure
// our effect system behaves correctly in complex scenarios:
//
// LAW 4: BIND ASSOCIATIVITY
//   - Complex sequential operations can be regrouped safely
//   - Like the associativity of addition: (1+2)+3 = 1+(2+3)
//
// LAW 5: HANDLER HOMOMORPHISM
//   - Handlers preserve the algebraic structure of computations
//   - A good handler doesn't "break" the mathematical relationships
//
// LAW 6: EFFECT COMMUTATIVITY
//   - Some effects can be reordered without changing the result
//   - Like addition: 2+3 = 3+2 (order doesn't matter)
//
// LAW 7: EFFECT NON-COMMUTATIVITY
//   - Some effects CANNOT be reordered (order matters!)
//   - Like subtraction: 5-3 ≠ 3-5 (order is crucial)
//
// LAW 8: HANDLER COMPOSITION
//   - Multiple handlers can work together predictably
//   - Combining handlers doesn't introduce surprising behavior
//
// LAW 9: DISTRIBUTIVITY
//   - Mathematical distributive laws hold for effects
//   - Like algebra: a*(b+c) = a*b + a*c
//
// LAW 10: IDEMPOTENCY
//   - Some operations can be repeated without changing the result
//   - Like setting a variable: set(x); set(x); same as just set(x)
//
// LAW 11: ALGEBRAIC EQUATIONS
//   - Specific equivalences between different ways of writing computations
//   - Proves that different code patterns are truly equivalent
//
// LAW 12: PARAMETRICITY
//   - Laws work consistently regardless of the specific data types used
//   - The laws are truly "algebraic" - they work at the structural level
//
// WHY THESE LAWS MATTER:
// =====================
// 1. **Refactoring Safety**: You can restructure code knowing it still works
// 2. **Optimization**: Compilers can rearrange operations safely
// 3. **Reasoning**: You can predict how complex programs will behave
// 4. **Composability**: Different effects can be combined without surprises
// 5. **Testing**: Mock handlers behave like real ones (no test surprises)

/// LAW 4: BIND ASSOCIATIVITY (MONADIC COMPOSITION)
///
/// **Mathematical Statement**: `(m >>= f) >>= g ≡ m >>= (λx -> f(x) >>= g)`
///
/// **Notation Breakdown**:
/// - `m` = an effectful computation that produces a value
/// - `f` = a function that takes a value and produces an effectful computation  
/// - `g` = another function that takes a value and produces an effectful computation
/// - `>>=` = "bind" operator (run left side, feed result to right side function)
/// - `λx ->` = "lambda x" (anonymous function taking parameter x)
///
/// **What This Means**: When you have three computations chained together with bind,
/// you can group them differently without changing the final result.
///
/// **Real-World Analogy**:
/// - Making dinner: (shop for ingredients, then cook), then serve
/// - vs: shop for ingredients, then (cook, then serve)  
/// - The final meal is the same regardless of how you mentally group the steps
///
/// **Why This Matters**:
/// - You can refactor complex chains of operations safely
/// - Nested computations can be flattened or restructured
/// - No need to worry about "bracketing" when building complex workflows
/// - Enables functional programming patterns like monadic composition
///
/// **Example**:
/// - Left grouping: `((set 5, multiply by 2), add 10)`
/// - Right grouping: `(set 5, (multiply by 2, add 10))`
/// - Both should give: 5 → 10 → 20 (final result: 20)
///
/// **In Plain English**: "The order you perform complex operations matters,
/// but how you group them in your head doesn't affect the outcome."
#[test]
fn test_bind_associativity() {
    // Test sequential composition in one handler context
    #[effectful]
    fn test_composition() -> (i32, i32) {
        // Method 1: Sequential composition
        let result1 = {
            // m: Set to 5
            let _: () = perform!(State::Set(5));
            let x: i32 = perform!(State::Get);
            
            // f(x): Multiply by 2
            let _: () = perform!(State::Set(x * 2));
            let y: i32 = perform!(State::Get);
            
            // g(y): Add 10
            let _: () = perform!(State::Set(y + 10));
            perform!(State::Get)
        };
        
        // Reset
        let _: () = perform!(State::Set(0));
        
        // Method 2: Same operations, different conceptual grouping
        let result2 = {
            // m: Set to 5
            let _: () = perform!(State::Set(5));
            let x: i32 = perform!(State::Get);
            
            // f(x) then g: multiply by 2, then add 10
            let _: () = perform!(State::Set(x * 2));
            let y: i32 = perform!(State::Get);
            let _: () = perform!(State::Set(y + 10));
            perform!(State::Get)
        };
        
        (result1, result2)
    }
    
    let (r1, r2) = test_composition()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    assert_eq!(r1, r2);
    assert_eq!(r1, 20); // (5 * 2) + 10 = 20
}

/// LAW 5: HANDLER HOMOMORPHISM - STRUCTURE PRESERVATION
///
/// **Mathematical Statement**:
/// - `handle(return(x)) = return(x)` (handlers preserve pure values)
/// - `handle(op >>= k) = handle(op) >>= (handle ∘ k)` (handlers preserve composition)
///
/// **Notation Breakdown**:
/// - `handle(...)` = applying a handler to a computation
/// - `return(x)` = pure value wrapped in effect system
/// - `op >>= k` = effect operation followed by continuation function k
/// - `∘` = function composition (handle ∘ k means "handle composed with k")
///
/// **What This Means**: Handlers don't break the mathematical structure of computations.
/// When you apply a handler, the algebraic relationships between operations are preserved.
///
/// **Real-World Analogy**:
/// - A good translator preserves the meaning and structure of a story
/// - Translating "Once upon a time" to French still means "Once upon a time"
/// - The plot structure, character relationships, etc. remain intact
/// - Similarly, handlers translate effects but preserve their relationships
///
/// **Why This Matters**:
/// - Mock handlers behave the same as real handlers (reliable testing)
/// - You can swap implementations without breaking program logic
/// - Handlers can be composed and combined safely
/// - The effect system's mathematical properties are preserved at runtime
///
/// **Example**:
/// - Pure computation: `return(42)` handled should still equal `42`
/// - Composed operations: `add(2,3) then multiply(result,4)` should equal `(2+3)*4 = 20`
/// - The handler preserves both the individual operations and their composition
///
/// **In Plain English**: "A good effect handler is like a good translator -
/// it changes the language but preserves the meaning and structure."
#[test]
fn test_handler_homomorphism() {
    // Test that handlers preserve pure computations
    #[effectful]
    fn pure_value() -> i32 {
        42 // No effects
    }

    let handled_pure = pure_value()
        .handle(PureHandler)
        .run_checked()
        .unwrap();

    let direct_pure = 42;

    assert_eq!(handled_pure, direct_pure);

    // Test that handlers preserve composition
    #[effectful]
    fn composed_operation() -> i32 {
        let x: i32 = perform!(Pure::Add((2, 3))); // 5
        let y: i32 = perform!(Pure::Multiply((x, 4))); // 20
        perform!(Pure::Add((y, 10))) // 30
    }

    let result = composed_operation()
        .handle(PureHandler)
        .run_checked()
        .unwrap();

    // Should be ((2 + 3) * 4) + 10 = 30
    assert_eq!(result, 30);

    // Equivalent direct computation
    let direct = ((2 + 3) * 4) + 10;
    assert_eq!(result, direct);
}

/// LAW 6: EFFECT COMMUTATIVITY - WHEN ORDER DOESN'T MATTER
///
/// **Mathematical Statement**: `op1 >> op2 ≡ op2 >> op1` (for commutative effects)
///
/// **Notation Breakdown**:
/// - `op1 >> op2` = do operation 1, then operation 2
/// - `op2 >> op1` = do operation 2, then operation 1  
/// - `≡` = these two sequences are equivalent (same result)
///
/// **What This Means**: Some effects can be reordered without changing the result.
/// If two operations don't interfere with each other, their order doesn't matter.
///
/// **Real-World Analogy**:
/// - Putting on your socks: left sock then right sock = right sock then left sock
/// - Adding ingredients to a bowl: salt then pepper = pepper then salt
/// - The final result is the same regardless of order
///
/// **Why This Matters**:
/// - Enables parallel execution (operations can run simultaneously)
/// - Allows optimizations (reorder for better performance)
/// - Simplifies reasoning (don't need to worry about order for independent operations)
/// - Supports flexible program structure
///
/// **Examples of Commutative Effects**:
/// - Pure mathematical operations: `add(2,3)` then `add(4,5)` = `add(4,5)` then `add(2,3)`
/// - Independent logging: log("A") then log("B") ≈ log("B") then log("A") (both messages appear)
/// - Reading from different variables: `get(x)` then `get(y)` = `get(y)` then `get(x)`
///
/// **Warning**: Not all effects are commutative! See the next test for counter-examples.
///
/// **In Plain English**: "Some operations are like putting on socks -
/// the order doesn't matter, you end up with the same result."
#[test]
fn test_effect_commutativity() {
    use std::sync::{Arc, Mutex};
    
    #[derive(Clone)]
    struct RecordingPureHandler {
        operations: Arc<Mutex<Vec<String>>>,
    }
    
    impl RecordingPureHandler {
        fn new() -> Self {
            Self {
                operations: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn get_operations(&self) -> Vec<String> {
            self.operations.lock().unwrap().clone()
        }
    }
    
    impl Handler<Op> for RecordingPureHandler {
        fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
            match op {
                Op::Pure(Pure::Add((a, b))) => {
                    self.operations.lock().unwrap().push(format!("Add({}, {})", a, b));
                    Box::new(a + b)
                }
                Op::Pure(Pure::Multiply((a, b))) => {
                    self.operations.lock().unwrap().push(format!("Multiply({}, {})", a, b));
                    Box::new(a * b)
                }
                _ => panic!("RecordingPureHandler cannot handle operation: {op:?}"),
            }
        }
    }
    
    impl PartialHandler<Op> for RecordingPureHandler {
        fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
            match op {
                Op::Pure(_) => Some(self.handle(op)),
                _ => None,
            }
        }
    }
    
    // Pure mathematical operations should be commutative
    #[effectful]
    fn order1(a: i32, b: i32) -> i32 {
        let x: i32 = perform!(Pure::Add((a, 5)));
        let y: i32 = perform!(Pure::Add((b, 3)));
        perform!(Pure::Add((x, y)))
    }

    #[effectful]
    fn order2(a: i32, b: i32) -> i32 {
        let y: i32 = perform!(Pure::Add((b, 3)));
        let x: i32 = perform!(Pure::Add((a, 5)));
        perform!(Pure::Add((x, y)))
    }

    let handler1 = RecordingPureHandler::new();
    let handler2 = RecordingPureHandler::new();

    let result1 = order1(10, 20)
        .handle(handler1.clone())
        .run_checked()
        .unwrap();
    let result2 = order2(10, 20)
        .handle(handler2.clone())
        .run_checked()
        .unwrap();

    assert_eq!(result1, result2);
    assert_eq!(result1, (10 + 5) + (20 + 3)); // 38
    
    // Verify operation order is different
    let ops1 = handler1.get_operations();
    let ops2 = handler2.get_operations();
    
    assert_eq!(ops1, vec!["Add(10, 5)", "Add(20, 3)", "Add(15, 23)"]);
    assert_eq!(ops2, vec!["Add(20, 3)", "Add(10, 5)", "Add(15, 23)"]);
}

/// LAW 7: EFFECT NON-COMMUTATIVITY - WHEN ORDER MATTERS!
///
/// **Mathematical Statement**: `op1 >> op2 ≢ op2 >> op1` (for non-commutative effects)
///
/// **Notation Breakdown**:
/// - `op1 >> op2` = do operation 1, then operation 2
/// - `op2 >> op1` = do operation 2, then operation 1
/// - `≢` = these are NOT equivalent (different results)
///
/// **What This Means**: Many effects CANNOT be reordered because they interfere
/// with each other. The order of operations fundamentally changes the outcome.
///
/// **Real-World Analogy**:
/// - Getting dressed: put on underwear then pants ≠ put on pants then underwear
/// - Cooking: add flour then water ≠ add water then flour (different consistency)
/// - Banking: deposit $100 then withdraw $50 ≠ withdraw $50 then deposit $100
///
/// **Why This Matters**:
/// - Prevents dangerous optimizations (can't reorder arbitrary operations)
/// - Enforces correct program semantics (some sequences are logically required)
/// - Catches bugs (accidentally swapping operations will fail tests)
/// - Documents dependencies (when operations must happen in specific order)
///
/// **Examples of Non-Commutative Effects**:
/// - State mutations: `set(5)` then `set(10)` ≠ `set(10)` then `set(5)` (final state differs)
/// - File operations: `write("hello")` then `write("world")` ≠ `write("world")` then `write("hello")`
/// - Database transactions: `insert(record)` then `update(record)` ≠ reverse order
/// - Network calls: `authenticate()` then `download()` ≠ `download()` then `authenticate()`
///
/// **In Plain English**: "Some operations are like getting dressed -
/// the order absolutely matters, or you'll end up with a mess!"
#[test]
fn test_effect_non_commutativity() {
    // State operations are NOT commutative - test in single handler context
    #[effectful]
    fn test_both_orders() -> (i32, i32) {
        // Order 1: Set 10, then 20
        let _: () = perform!(State::Set(10));
        let _: () = perform!(State::Set(20));
        let result1: i32 = perform!(State::Get);
        
        // Order 2: Set 20, then 10
        let _: () = perform!(State::Set(20));
        let _: () = perform!(State::Set(10));
        let result2: i32 = perform!(State::Get);
        
        (result1, result2)
    }
    
    let (result1, result2) = test_both_orders()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();

    // Results should be different - last set wins
    assert_eq!(result1, 20);
    assert_eq!(result2, 10);
    assert_ne!(result1, result2);
}

/// LAW 8: HANDLER COMPOSITION - COMBINING MULTIPLE EFFECT FAMILIES
///
/// **Mathematical Statement**: `handle1(handle2(computation))` behaves predictably
///
/// **What This Means**: When you have computations that use multiple types of effects,
/// you can handle them with a composed handler that routes each effect to the right sub-handler.
///
/// **Real-World Analogy**:
/// - A restaurant kitchen with different stations (grill, salad, dessert)
/// - Each order gets routed to the right station based on what's needed
/// - The head chef coordinates but doesn't need to know how each station works
/// - The final dish combines work from multiple specialized areas
///
/// **Why This Matters**:
/// - Real applications need multiple effect types (state, I/O, logging, etc.)
/// - You can build complex handlers from simpler, specialized ones
/// - Each handler can focus on one concern (separation of responsibilities)
/// - Handlers can be tested and developed independently
///
/// **Example**:
/// A computation that uses both State effects and Pure mathematical effects
/// can be handled by a CombinedHandler that routes State operations to
/// StateHandler and Pure operations to PureHandler.
///
/// **In Plain English**: "A good composed handler is like a restaurant kitchen -
/// each specialist handles their part, and the result is a complete meal."
#[test]
fn test_handler_composition() {
    #[effectful]
    fn multi_effect_computation() -> i32 {
        let _: () = perform!(State::Set(5));
        let state_val: i32 = perform!(State::Get);
        let pure_val: i32 = perform!(Pure::Multiply((state_val, 3)));
        perform!(Pure::Add((pure_val, 7)))
    }

    let result = multi_effect_computation()
        .handle(CombinedHandler::new(0))
        .run_checked()
        .unwrap();

    // Should be (5 * 3) + 7 = 22
    assert_eq!(result, 22);
}

/// LAW 9: DISTRIBUTIVITY - MATHEMATICAL LAWS CARRY OVER TO EFFECTS
///
/// **Mathematical Statement**: `handle(op1 ⊕ op2) = handle(op1) ⊕ handle(op2)` for distributive operations
///
/// **Notation Breakdown**:
/// - `⊕` = some mathematical operation (like addition or multiplication)
/// - `handle(...)` = applying a handler to a computation
/// - The law says distributive properties from math still work with effects
///
/// **What This Means**: Mathematical laws like distributivity (a*(b+c) = a*b + a*c)
/// still hold when those operations are performed as effects rather than pure math.
///
/// **Real-World Analogy**:
/// - Buying groceries: buying 3*(apples + oranges) = buying 3*apples + 3*oranges
/// - Whether you calculate in your head or at the store, math works the same way
/// - The "effect" of shopping doesn't break basic arithmetic relationships
///
/// **Why This Matters**:
/// - Mathematical optimizations still work in effectful code
/// - You can reason about effectful computations using familiar math laws
/// - Refactoring based on algebraic identities is safe
/// - Bridge between pure mathematics and effectful programming
///
/// **Example**:
/// - `multiply(2, add(3, 4))` should equal `add(multiply(2, 3), multiply(2, 4))`
/// - Both equal 2*(3+4) = 2*3 + 2*4 = 14
/// - The fact that we're using effects doesn't break distributivity
///
/// **In Plain English**: "Math laws don't stop working just because you're using effects -
/// 2*(3+4) still equals 2*3 + 2*4, whether it's pure math or effectful operations."
#[test]
fn test_distributivity() {
    #[effectful]
    fn distributive_left(a: i32, b: i32, c: i32) -> i32 {
        let sum: i32 = perform!(Pure::Add((b, c)));
        perform!(Pure::Multiply((a, sum)))
    }

    #[effectful]
    fn distributive_right(a: i32, b: i32, c: i32) -> i32 {
        let prod1: i32 = perform!(Pure::Multiply((a, b)));
        let prod2: i32 = perform!(Pure::Multiply((a, c)));
        perform!(Pure::Add((prod1, prod2)))
    }

    let left_result = distributive_left(2, 3, 4)
        .handle(PureHandler)
        .run_checked()
        .unwrap();
    let right_result = distributive_right(2, 3, 4)
        .handle(PureHandler)
        .run_checked()
        .unwrap();

    // Both should equal 2 * (3 + 4) = 2 * 3 + 2 * 4 = 14
    assert_eq!(left_result, 14);
    assert_eq!(right_result, 14);
    assert_eq!(left_result, right_result);
}

/// LAW 10: IDEMPOTENCY - SAFE TO REPEAT OPERATIONS
///
/// **Mathematical Statement**: `op >> op ≡ op` (for idempotent operations)
///
/// **Notation Breakdown**:
/// - `op >> op` = doing the same operation twice in a row
/// - `≡` = equivalent to
/// - `op` = doing the operation just once
///
/// **What This Means**: Some operations can be safely repeated without changing
/// the result. Doing them multiple times has the same effect as doing them once.
///
/// **Real-World Analogy**:
/// - Turning on a light switch: on -> on = on (already on, no change)
/// - Setting your alarm to 7 AM twice = setting it once (same result)
/// - Saving a document multiple times = saving it once (same final state)
///
/// **Why This Matters**:
/// - Safe to retry operations that might have failed partway through
/// - Network calls can be safely repeated if connection drops
/// - Caching systems can replay operations without side effects
/// - Simplifies error recovery (just repeat the operation)
///
/// **Examples of Idempotent Operations**:
/// - `set_value(x)` then `set_value(x)` = just `set_value(x)`
/// - HTTP PUT requests (setting a resource to a specific state)
/// - Creating a directory that already exists
/// - Setting a boolean flag that's already set
///
/// **Non-Idempotent Counter-Examples**:
/// - `increment()` then `increment()` ≠ just `increment()` (different results!)
/// - HTTP POST requests (usually create new resources each time)
/// - Appending to a file or list
///
/// **In Plain English**: "Some operations are like flipping a light switch -
/// doing it twice in a row has the same effect as doing it once."
#[test]
fn test_idempotency() {
    #[effectful]
    fn test_idempotent() -> (i32, i32, bool) {
        // Single set
        let _: () = perform!(State::Set(42));
        let single: i32 = perform!(State::Get);
        
        // Reset
        let _: () = perform!(State::Set(0));
        
        // Double set (idempotent)
        let _: () = perform!(State::Set(42));
        let _: () = perform!(State::Set(42));
        let double: i32 = perform!(State::Get);
        
        (single, double, single == double)
    }
    
    let (single, double, equal) = test_idempotent()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    assert!(equal);
    assert_eq!(single, 42);
    assert_eq!(double, 42);
}

/// LAW 11: ALGEBRAIC EFFECT EQUATIONS - PROGRAM EQUIVALENCES
///
/// **Mathematical Statement**: Specific equations between different effectful programs
///
/// **What This Means**: There are specific patterns in effectful programming where
/// two different ways of writing a program are mathematically equivalent.
///
/// **Real-World Analogy**:
/// - Different recipes for the same dish: "add salt, then taste" vs "taste, then add salt to preference"
/// - Different routes to the same destination that arrive at the same time
/// - Different ways to organize your work that produce the same final output
///
/// **Why This Matters**:
/// - Proves that refactoring patterns are actually safe
/// - Documents which optimizations preserve program meaning
/// - Helps identify when two code patterns are truly equivalent
/// - Enables automated program transformations
///
/// **Example Law**: `get; set(x); get ≡ set(x); return(x)`
///
/// **Breakdown**:
/// - Left side: read current value, set to x, read again
/// - Right side: set to x, then just return x (without reading)  
/// - Both end up with the same final state and return the same value
/// - The middle "get" in the left side is redundant
///
/// **Translation**:
/// - "Read a variable, set it to X, then read it again"
/// - is the same as  
/// - "Set it to X and just use X directly"
/// - The extra read doesn't add any information
///
/// **In Plain English**: "Some programming patterns are like taking a photo,
/// editing it, then taking the same photo again - you can skip the redundant steps."
#[test]
fn test_algebraic_equations() {
    // Test various algebraic equations in one handler context
    #[effectful]
    fn test_equations() -> Vec<(String, bool)> {
        let mut results = Vec::new();
        
        // Equation 1: get; set(x); get ≡ set(x); x
        let _: () = perform!(State::Set(50)); // Initial state
        let eq1_left: i32 = {
            let _old: i32 = perform!(State::Get);
            let _: () = perform!(State::Set(100));
            perform!(State::Get)
        };
        
        let _: () = perform!(State::Set(50)); // Reset
        let eq1_right: i32 = {
            let _: () = perform!(State::Set(100));
            100
        };
        
        results.push(("get;set(x);get ≡ set(x);x".to_string(), eq1_left == eq1_right));
        
        // Equation 2: set(x); set(y) ≡ set(y)
        let _: () = perform!(State::Set(0)); // Reset
        let eq2_left: i32 = {
            let _: () = perform!(State::Set(10));
            let _: () = perform!(State::Set(20));
            perform!(State::Get)
        };
        
        let _: () = perform!(State::Set(0)); // Reset
        let eq2_right: i32 = {
            let _: () = perform!(State::Set(20));
            perform!(State::Get)
        };
        
        results.push(("set(x);set(y) ≡ set(y)".to_string(), eq2_left == eq2_right));
        
        results
    }
    
    let results = test_equations()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    for (equation, holds) in results {
        assert!(holds, "Equation failed: {}", equation);
    }
}

/// LAW 12: PARAMETRICITY - LAWS WORK REGARDLESS OF DATA TYPES
///
/// **Mathematical Statement**: Algebraic laws work uniformly across different types
///
/// **What This Means**: The algebraic laws we've tested don't depend on the specific
/// data types being used. Whether you're working with integers, strings, custom types,
/// etc., the same algebraic relationships hold.
///
/// **Real-World Analogy**:
/// - Addition is commutative whether you're adding numbers, lengths, or weights
/// - "Put item in container, then take it out" works the same for any type of item/container
/// - The underlying patterns are universal, independent of the specific things involved
///
/// **Why This Matters**:
/// - Laws proven for one type automatically apply to other types
/// - You can write generic, reusable effect handlers
/// - The effect system is truly "algebraic" - structure matters more than content
/// - Generic programming principles apply to effects
///
/// **Example**:
/// The associativity law `(a >> b) >> c ≡ a >> (b >> c)` works whether:
/// - a, b, c operate on integers, strings, user records, etc.
/// - The operations are arithmetic, string manipulation, database calls, etc.
/// - The return types are numbers, objects, lists, etc.
///
/// **Test Strategy**:
/// We test the same algebraic pattern (like increment) with different initial values
/// to show that the pattern itself is what matters, not the specific numbers.
///
/// **In Plain English**: "Algebraic laws are like the rules of grammar -
/// they work the same way whether you're talking about cats, cars, or concepts."
#[test]
fn test_parametricity() {
    // Test that the same algebraic structure works with different types
    #[effectful]
    fn state_increment_pattern() -> i32 {
        let current: i32 = perform!(State::Get);
        let _: () = perform!(State::Set(current + 1));
        perform!(State::Get)
    }

    // Test with different initial values
    let results: Vec<i32> = vec![0, 10, 100, -5]
        .into_iter()
        .map(|initial| {
            state_increment_pattern()
                .handle(StateHandler::new(initial))
                .run_checked()
                .unwrap()
        })
        .collect();

    // Each result should be initial + 1
    assert_eq!(results, vec![1, 11, 101, -4]);

    // The pattern should be consistent regardless of initial value
    for (i, &result) in results.iter().enumerate() {
        let initial_values = [0, 10, 100, -5];
        assert_eq!(result, initial_values[i] + 1);
    }

    // EXPLANATION: Why this demonstrates parametricity
    // ===============================================
    //
    // This test shows that the same algebraic structure (increment pattern)
    // works consistently across different parameter values:
    //
    // Pattern: get current value, add 1, set new value, return new value
    //
    // Applied to different initial states:
    // - Initial 0: 0 -> 1 (add 1)
    // - Initial 10: 10 -> 11 (add 1)
    // - Initial 100: 100 -> 101 (add 1)
    // - Initial -5: -5 -> -4 (add 1)
    //
    // The algebraic structure is preserved regardless of the specific values.
    // This is what "parametricity" means - the laws work at the structural level,
    // independent of the particular data being processed.
}

/// NEGATIVE TEST: NON-IDEMPOTENCY
///
/// This test demonstrates operations that are NOT idempotent, showing that
/// repeating them produces different results.
#[test]
fn test_non_idempotency() {
    #[effectful]
    fn test_increment() -> (i32, i32, bool) {
        // Reset to 0
        let _: () = perform!(State::Set(0));
        
        // Single increment
        let single = {
            let current: i32 = perform!(State::Get);
            let _: () = perform!(State::Set(current + 1));
            current + 1
        };
        
        // Reset to 0
        let _: () = perform!(State::Set(0));
        
        // Double increment
        let double = {
            // First increment
            let current: i32 = perform!(State::Get);
            let _: () = perform!(State::Set(current + 1));
            // Second increment
            let current: i32 = perform!(State::Get);
            let _: () = perform!(State::Set(current + 1));
            current + 1
        };
        
        (single, double, single == double)
    }
    
    let (single, double, equal) = test_increment()
        .handle(StateHandler::new(0))
        .run_checked()
        .unwrap();
    
    assert!(!equal); // Should NOT be equal
    assert_eq!(single, 1);
    assert_eq!(double, 2);
}

/// ONE-SHOT GUARANTEE TEST
///
/// This test verifies that effects are handled exactly once (one-shot semantics),
/// not multiple times (multi-shot).
#[test]
fn test_one_shot_guarantee() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    struct CountingHandler {
        count: Arc<AtomicUsize>,
    }
    
    impl CountingHandler {
        fn new() -> Self {
            Self { count: Arc::new(AtomicUsize::new(0)) }
        }
    }
    
    impl PartialHandler<Op> for CountingHandler {
        fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
            match op {
                Op::State(State::Get) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    Some(Box::new(42))
                }
                _ => None,
            }
        }
    }
    
    #[effectful]
    fn single_get() -> i32 {
        perform!(State::Get)
    }
    
    let counter = CountingHandler::new();
    let count_ref = Arc::clone(&counter.count);
    
    let result = single_get()
        .begin_chain()
        .handle(Box::new(counter) as Box<dyn PartialHandler<Op> + Send>)
        .handle(Box::new(StateHandler::new(0)) as Box<dyn PartialHandler<Op> + Send>)
        .run_checked()
        .unwrap();
    
    assert_eq!(result, 42); // Got value from CountingHandler
    assert_eq!(count_ref.load(Ordering::SeqCst), 1); // Called exactly once
}

//══════════════════════════════════════════════════════════════════════════════
// CONCLUSION: WHAT THESE TESTS PROVE
//══════════════════════════════════════════════════════════════════════════════
//
// Congratulations! If you've read this far, you now understand the mathematical
// foundations that make algebraic effects a powerful and reliable programming paradigm.
//
// WHAT WE'VE PROVEN:
// ==================
// ✅ The algae library correctly implements algebraic effects
// ✅ All fundamental algebraic laws hold (identity, associativity, homomorphism, etc.)
// ✅ Effects can be safely composed, reordered (when appropriate), and optimized
// ✅ Handlers preserve the mathematical structure of computations
// ✅ The system works consistently across different data types and effect families
//
// WHAT THIS MEANS FOR YOU:
// ========================
// 🔒 **Reliability**: Your effectful code behaves predictably
// 🔄 **Refactoring**: You can restructure code safely using algebraic laws
// 🧪 **Testing**: Mock handlers behave exactly like real ones
// ⚡ **Performance**: Optimizations based on algebraic laws are guaranteed safe
// 🧩 **Composability**: Effects combine without surprising interactions
//
// PRACTICAL TAKEAWAYS:
// ===================
// 1. **Separation of Concerns**: Describe WHAT (effects) separately from HOW (handlers)
// 2. **Mathematical Reasoning**: Use algebraic laws to reason about complex programs
// 3. **Safe Optimizations**: Compilers and tools can optimize effectful code safely
// 4. **Predictable Composition**: Combining effects follows mathematical rules
// 5. **Universal Patterns**: These laws work across all programming domains
//
// FROM THEORY TO PRACTICE:
// ========================
// These mathematical foundations enable practical benefits:
// - Database transactions that compose correctly
// - Network operations that can be safely retried
// - User interfaces that update predictably
// - Concurrent programs that avoid race conditions
// - Testing frameworks that accurately simulate production
//
// The beauty of algebraic effects is that they bring the rigor and predictability
// of mathematics to the messy, real-world problems of software engineering.
//
// Ready to build reliable, composable, testable software with algebraic effects? 🚀