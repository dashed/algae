//! # Algae Macros - Procedural Macros for Algebraic Effects
//!
//! This crate provides the procedural macros that power the algae algebraic effects library.
//! It includes macros for defining effects, marking functions as effectful, and performing
//! effect operations.
//!
//! ## One-Shot (Linear) Effects
//!
//! The macros in this crate generate code for **one-shot (linear) algebraic effects**:
//! - Each `perform!` operation receives exactly one response
//! - Effects cannot be resumed or replayed multiple times  
//! - Linear control flow ensures predictable execution
//! - Simpler implementation with better performance than multi-shot alternatives
//!
//! ## Provided Macros
//!
//! - [`effect!`] - Defines effect families and operations
//! - [`effectful`] - Transforms functions into effectful computations  
//! - [`perform!`] - Performs effect operations within effectful functions
//!
//! These macros are typically used through the `algae::prelude` module rather than directly.
//!
//! ## Example Usage
//!
//! ```ignore
//! #![feature(coroutines, coroutine_trait, yield_expr)]
//! use algae::prelude::*;
//!
//! // Define effects using the effect! macro
//! effect! {
//!     Console::Print (String) -> ();
//!     Console::ReadLine -> String;
//! }
//!
//! // Create effectful functions using the #[effectful] attribute
//! #[effectful]
//! fn greet() -> String {
//!     let _: () = perform!(Console::Print("Hello!".to_string()));
//!     let name: String = perform!(Console::ReadLine);
//!     format!("Nice to meet you, {}!", name)
//! }
//! ```
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, punctuated::Punctuated, Ident, Result,
    Token, Type,
};

/*──────────────────────────────────────────────────────────────────────────────
  effect!{ … }
  Grammar (semicolon or comma separators, line‑breaks irrelevant):

    effect! {
        Family::Variant            -> Ret;
        Family::Variant (Payload)  -> Ret,
        …
    }

  The "(Payload)" part may be omitted when there is no payload.
  If you keep it, it can be the empty tuple "()".
  `Ret` is parsed and stored but only used for documentation – the run‑time
  uses dynamic down‑casting to recover it.

  The expansion is roughly:

    pub enum Family { Variant(Payload), … }
    pub enum Op     { Family(Family), … }

    impl From<Family> for Op { … }   // one per family
──────────────────────────────────────────────────────────────────────────────*/

/// One operation line:  `Family::Variant (Payload?) -> Ret`
struct OpLine {
    family: Ident,
    variant: Ident,
    payload: Option<Type>,
    _arrow: Token![->],
    _ret: Type,
}

impl Parse for OpLine {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let family: Ident = input.parse()?;
        input.parse::<Token![::]>()?;
        let variant: Ident = input.parse()?;

        // optional payload in parentheses
        let payload = if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            if content.is_empty() {
                None
            } else {
                Some(content.parse::<Type>()?)
            }
        } else {
            None
        };

        let arrow: Token![->] = input.parse()?;
        let ret: Type = input.parse()?;

        Ok(Self {
            family,
            variant,
            payload,
            _arrow: arrow,
            _ret: ret,
        })
    }
}

/// The whole macro input – optional root header plus list of OpLines separated by `;` or `,`.
struct EffectInput {
    root_ident: Option<Ident>,
    lines: Punctuated<OpLine, Token![;]>, // accept `;`  – we strip trailing ones.
}

impl Parse for EffectInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // Try to parse optional "root EnumName;" header
        let root_ident = if input.peek(syn::Ident) {
            // Fork the input to check if this starts with "root"
            let fork = input.fork();
            if let Ok(ident) = fork.parse::<Ident>() {
                if ident == "root" {
                    // Consume the "root" keyword
                    let _root_kw: Ident = input.parse()?;
                    // Parse the root enum name
                    let root_name: Ident = input.parse()?;
                    // Consume the semicolon
                    input.parse::<Token![;]>()?;
                    Some(root_name)
                } else {
                    // This is just a regular effect line starting with Family::
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        let lines = Punctuated::<OpLine, Token![;]>::parse_terminated(input)?;
        Ok(Self { root_ident, lines })
    }
}

/// Defines effect families and their operations.
///
/// The `effect!` macro is used to declare the effects that your program can perform.
/// It generates enums for each effect family and a unified root enum (default `Op`) that combines
/// all families. Each operation specifies its parameter types and return type.
///
/// ## Custom Root Enum Names
///
/// You can specify a custom name for the root enum using the `root EnumName;` syntax.
/// This allows multiple `effect!` declarations in the same module without conflicts:
///
/// # Syntax
///
/// ```text
/// // Default syntax (root enum named "Op")
/// effect! {
///     Family::Operation (ParameterType) -> ReturnType;
///     Family::Operation -> ReturnType;  // No parameters
///     Family::Operation (TupleType) -> ReturnType;  // Multiple parameters as tuple
/// }
///
/// // Custom root enum name
/// effect! {
///     root CustomOp;
///     Family::Operation (ParameterType) -> ReturnType;
///     Family::Operation -> ReturnType;
/// }
/// ```
///
/// ## Multiple Effects in Same Module
///
/// The `root EnumName;` syntax prevents conflicts when multiple `effect!` declarations
/// exist in the same module:
///
/// ```ignore
/// # use algae::prelude::*;
/// // First effect with custom root
/// effect! {
///     root ConsoleOp;
///     Console::Print (String) -> ();
///     Console::ReadLine -> String;
/// }
///
/// // Second effect with different custom root
/// effect! {
///     root FileOp;
///     File::Read (String) -> String;
///     File::Write ((String, String)) -> ();
/// }
/// ```
///
/// Without custom root names, the above would cause a compilation error due to
/// duplicate `Op` enum definitions.
///
/// # Generated Code
///
/// For each effect family, this macro generates:
/// - A family enum with variants for each operation
/// - A unified root enum (default `Op` or custom name) that contains all families
/// - `From` implementations to convert family enums to the root enum
/// - `Default` implementations where applicable
/// - Debug derive implementations
/// - A hidden sentry enum to detect duplicate root names
///
/// # Examples
///
/// ## Basic Effects
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// effect! {
///     Console::Print (String) -> ();
///     Console::ReadLine -> String;
/// }
/// 
/// // Generates:
/// // enum Console { Print(String), ReadLine }
/// // enum Op { Console(Console) }
/// ```
///
/// ## Multiple Effect Families
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// effect! {
///     File::Read (String) -> String;
///     File::Write ((String, String)) -> ();  // Tuple for multiple params
///     
///     Network::Get (String) -> Result<String, String>;
///     Network::Post ((String, String)) -> Result<String, String>;
/// }
/// 
/// // Generates:
/// // enum File { Read(String), Write((String, String)) }
/// // enum Network { Get(String), Post((String, String)) }
/// // enum Op { File(File), Network(Network) }
/// ```
///
/// ## Complex Types
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # use std::collections::HashMap;
/// effect! {
///     Database::Query (String) -> Vec<HashMap<String, String>>;
///     Database::Execute (String) -> Result<u64, String>;
/// }
/// ```
///
/// # Usage with Handlers
///
/// After defining effects, you implement handlers that process these operations:
///
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Console::Print (String) -> (); Console::ReadLine -> String; }
/// struct ConsoleHandler;
///
/// impl Handler<Op> for ConsoleHandler {
///     fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
///         match op {
///             Op::Console(Console::Print(msg)) => {
///                 println!("{}", msg);
///                 Box::new(())
///             }
///             Op::Console(Console::ReadLine) => {
///                 // In real code, read from stdin
///                 Box::new("user input".to_string())
///             }
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn effect(item: TokenStream) -> TokenStream {
    let EffectInput { root_ident, lines } = parse_macro_input!(item as EffectInput);
    
    // Determine the root enum name (default to "Op")
    let root_ident = root_ident.unwrap_or_else(|| Ident::new("Op", proc_macro2::Span::call_site()));
    
    // Generate sentry enum to catch duplicate root names
    let sentry_name = format!("__ALGAE_EFFECT_SENTRY_FOR_{root_ident}");
    let sentry_ident = Ident::new(&sentry_name, proc_macro2::Span::call_site());

    // ── 1.  Group lines by family ────────────────────────────────────────────
    #[derive(Clone)]
    struct VariantInfo {
        variant: Ident,
        payload: Option<Type>,
    }

    let mut families: BTreeMap<String, (Ident, Vec<VariantInfo>)> = BTreeMap::new();

    for l in lines {
        let entry = families
            .entry(l.family.to_string())
            .or_insert_with(|| (l.family.clone(), Vec::new()));
        entry.1.push(VariantInfo {
            variant: l.variant,
            payload: l.payload,
        });
    }

    // ── 2.  Generate one enum per family ─────────────────────────────────────
    let mut family_enums = TokenStream2::new();
    let mut op_variants = TokenStream2::new();
    let mut impl_froms = TokenStream2::new();

    // Get first family info before iterating
    let first_family = families.values().next().cloned();
    
    for (_fam_name_str, (family_ident, variants)) in families {
        // each variant
        let mut variant_tokens = TokenStream2::new();
        for v in &variants {
            let VariantInfo { variant, payload } = v;
            if let Some(ty) = payload {
                variant_tokens.extend(quote! { #variant(#ty), });
            } else {
                variant_tokens.extend(quote! { #variant, });
            }
        }

        // Create Default implementation for this family
        let first_variant = variants.first();
        let family_default = if let Some(first_variant) = first_variant {
            let variant = &first_variant.variant;
            if first_variant.payload.is_some() {
                quote! {
                    impl Default for #family_ident {
                        fn default() -> Self {
                            #family_ident::#variant(Default::default())
                        }
                    }
                }
            } else {
                quote! {
                    impl Default for #family_ident {
                        fn default() -> Self {
                            #family_ident::#variant
                        }
                    }
                }
            }
        } else {
            quote! {}
        };

        family_enums.extend(quote! {
            #[derive(Debug)]
            pub enum #family_ident {
                #variant_tokens
            }
            
            #family_default
        });

        // RootEnum::Family(Family)
        op_variants.extend(quote! { #family_ident(#family_ident), });

        impl_froms.extend(quote! {
            impl From<#family_ident> for #root_ident {
                fn from(f: #family_ident) -> Self { #root_ident::#family_ident(f) }
            }
        });
    }

    // ── 3.  Root enum (configurable name) ────────────────────────────────────
    
    // For Default implementation, we need to pick the first family and first variant
    let default_impl = if let Some((family_ident, variants)) = first_family {
        if let Some(first_variant) = variants.first() {
            let variant = &first_variant.variant;
            if first_variant.payload.is_some() {
                quote! {
                    impl Default for #root_ident {
                        fn default() -> Self {
                            #root_ident::#family_ident(#family_ident::#variant(Default::default()))
                        }
                    }
                }
            } else {
                quote! {
                    impl Default for #root_ident {
                        fn default() -> Self {
                            #root_ident::#family_ident(#family_ident::#variant)
                        }
                    }
                }
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };
    
    let output = quote! {
        // Sentry enum to detect duplicate root names in same module
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        enum #sentry_ident {}

        #family_enums

        #[derive(Debug)]
        pub enum #root_ident {
            #op_variants
        }

        #impl_froms
        
        #default_impl
    };

    output.into()
}

/*──────────────────────────────────────────────────────────────────────────────
   effectful!  and  perform!  are unchanged except for *one* tiny tweak:
   perform!( … ) now calls `.into()` so any Family enum is automatically
   promoted to `Op`.  The rest of the file is identical to what you had,
   minus the old stub.
──────────────────────────────────────────────────────────────────────────────*/

/// Transforms a function into an effectful computation.
///
/// The `#[effectful]` attribute macro converts a regular function into one that
/// returns an `Effectful<R, Op>` type, where `R` is the original return type.
/// The function body is transformed into a coroutine that can yield effects
/// using the `perform!` macro.
///
/// # Syntax
///
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Test::GetValue -> i32; }
/// #[effectful]
/// fn my_function() -> ReturnType {
///     // Function body can use perform! to perform effects
///     let value = perform!(SomeEffect::SomeOperation);
///     // ... rest of function
/// }
/// ```
///
/// # Transformation
///
/// The macro transforms the function in several ways:
/// 1. Changes return type from `T` to `Effectful<T, Op>`
/// 2. Wraps the function body in a coroutine generator
/// 3. Ensures the coroutine can yield `Effect<Op>` values
/// 4. Handles the resume/yield cycle for effect processing
///
/// # Examples
///
/// ## Simple Effectful Function
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Math::Add ((i32, i32)) -> i32; }
/// #[effectful]
/// fn add_numbers(a: i32, b: i32) -> i32 {
///     perform!(Math::Add((a, b)))
/// }
/// 
/// // This transforms to roughly:
/// // fn add_numbers(a: i32, b: i32) -> Effectful<i32, Op> {
/// //     Effectful::new(#[coroutine] |_| {
/// //         // ... coroutine implementation
/// //     })
/// // }
/// ```
///
/// ## Complex Effectful Function
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { 
/// #     State::Get -> i32; 
/// #     State::Set (i32) -> (); 
/// #     Logger::Info (String) -> (); 
/// # }
/// #[effectful]
/// fn complex_computation(initial: i32) -> String {
///     let _: () = perform!(Logger::Info("Starting computation".to_string()));
///     
///     let current: i32 = perform!(State::Get);
///     let _: () = perform!(State::Set(current + initial));
///     
///     let final_value: i32 = perform!(State::Get);
///     let _: () = perform!(Logger::Info(format!("Final value: {}", final_value)));
///     
///     format!("Result: {}", final_value)
/// }
/// ```
///
/// ## With Control Flow
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Random::Int (std::ops::Range<i32>) -> i32; Logger::Info (String) -> (); }
/// #[effectful]
/// fn random_process() -> String {
///     for i in 0..3 {
///         let value: i32 = perform!(Random::Int(1..10));
///         let _: () = perform!(Logger::Info(format!("Iteration {}: {}", i, value)));
///         
///         if value > 5 {
///             return format!("High value {} found early!", value);
///         }
///     }
///     "No high values found".to_string()
/// }
/// ```
///
/// # Usage
///
/// Effectful functions are called like normal functions but return an `Effectful`
/// type that must be run with a handler:
///
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Test::GetValue -> i32; }
/// # struct TestHandler;
/// # impl Handler<Op> for TestHandler {
/// #     fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
/// #         match op { Op::Test(Test::GetValue) => Box::new(42i32) }
/// #     }
/// # }
/// # #[effectful]
/// # fn my_computation() -> i32 { perform!(Test::GetValue) }
/// let result = my_computation()
///     .handle(TestHandler)
///     .run();
/// ```
///
/// # Limitations
///
/// - Functions must not be `async` (effectful functions use coroutines, not async/await)
/// - Generic parameters are preserved but may require careful handling with effects
/// - Lifetime parameters are supported but the coroutine has `'static` requirements
#[proc_macro_attribute]
pub fn effectful(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as syn::ItemFn);
    let body = &f.block;

    // Transform return type from -> T to -> Effectful<T, Op>
    let inner_type = match &f.sig.output {
        syn::ReturnType::Default => syn::parse_quote! { () },
        syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
    };
    
    f.sig.output = syn::parse_quote! {
        -> algae::Effectful<#inner_type, Op>
    };

    f.block = syn::parse_quote! {{
        algae::Effectful::new(#[coroutine] move |mut _reply: Option<algae::Reply>| {
            #body
        })
    }};
    quote!(#f).into()
}

/// Performs an effect operation within an effectful function.
///
/// The `perform!` macro is used inside functions marked with `#[effectful]` to
/// execute effect operations. It yields the effect to the handler, suspends the
/// coroutine, and resumes with the handler's reply value.
///
/// ## One-Shot Semantics
///
/// Each `perform!` operation follows algae's one-shot model:
/// - The effect is sent to the handler **exactly once**
/// - The handler provides **exactly one response**
/// - The computation resumes **exactly once** with that response
/// - No continuation capture or replay mechanisms are involved
///
/// # Syntax
///
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Test::GetValue -> i32; }
/// # #[effectful]
/// # fn example() -> i32 {
/// let result: ReturnType = perform!(Family::Operation(parameters));
/// # result
/// # }
/// ```
///
/// # Type Safety
///
/// The macro ensures type safety by:
/// 1. Converting the operation to the unified `Op` type using `Into`
/// 2. Creating an `Effect` with the operation
/// 3. Yielding the effect to the handler
/// 4. Extracting the reply with the correct type using `Reply::take()`
///
/// The return type must match what the effect definition specifies for that operation.
///
/// # Examples
///
/// ## Basic Usage
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { 
/// #     State::Get -> i32; 
/// #     State::Set (i32) -> (); 
/// # }
/// #[effectful]
/// fn state_example() -> i32 {
///     // Get current state (returns i32)
///     let current: i32 = perform!(State::Get);
///     
///     // Set new state (returns ())
///     let _: () = perform!(State::Set(current + 1));
///     
///     // Get updated state
///     let updated: i32 = perform!(State::Get);
///     updated
/// }
/// ```
///
/// ## With Complex Types
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { 
/// #     File::Read (String) -> Result<String, String>; 
/// #     Logger::Error (String) -> (); 
/// # }
/// #[effectful]
/// fn file_example(filename: String) -> String {
///     // Perform file read operation
///     let result: Result<String, String> = perform!(File::Read(filename.clone()));
///     
///     match result {
///         Ok(content) => content,
///         Err(error) => {
///             // Log the error
///             let _: () = perform!(Logger::Error(format!("Failed to read {}: {}", filename, error)));
///             "default content".to_string()
///         }
///     }
/// }
/// ```
///
/// ## Multiple Parameter Effects
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Math::Add ((i32, i32)) -> i32; }
/// #[effectful]
/// fn math_example() -> i32 {
///     // Multiple parameters are passed as a tuple
///     let sum: i32 = perform!(Math::Add((5, 3)));
///     sum
/// }
/// ```
///
/// ## Chaining Operations
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { 
/// #     Counter::Get -> i32; 
/// #     Counter::Increment -> (); 
/// #     Logger::Info (String) -> (); 
/// # }
/// #[effectful]
/// fn counter_example() -> i32 {
///     let initial: i32 = perform!(Counter::Get);
///     let _: () = perform!(Logger::Info(format!("Initial value: {}", initial)));
///     
///     let _: () = perform!(Counter::Increment);
///     let _: () = perform!(Counter::Increment);
///     
///     let final_value: i32 = perform!(Counter::Get);
///     let _: () = perform!(Logger::Info(format!("Final value: {}", final_value)));
///     
///     final_value
/// }
/// ```
///
/// # Generated Code
///
/// The `perform!` macro expands to coroutine code that:
/// 1. Creates an `Effect` from the operation
/// 2. Yields it to the effect runtime
/// 3. Waits for the handler to process it
/// 4. Extracts the typed result from the reply
///
/// ```ignore
/// # #![feature(coroutines, coroutine_trait, yield_expr)]
/// # use algae::prelude::*;
/// # effect! { Test::GetValue -> i32; }
/// // perform!(Test::GetValue) expands to roughly:
/// // {
/// //     let __eff = algae::Effect::new(Test::GetValue.into());
/// //     let __reply_opt = yield __eff;
/// //     __reply_opt.unwrap().take::<i32>()
/// // }
/// ```
///
/// # Error Handling
///
/// - **Type Mismatch**: If the handler returns the wrong type, `Reply::take()` will panic with a descriptive error
/// - **Missing Reply**: If the effect system fails to provide a reply, the macro will panic
/// - **Handler Errors**: Handlers should return appropriate error types (like `Result`) rather than panicking
///
/// # Usage Notes
///
/// - Can only be used inside `#[effectful]` functions
/// - The operation must implement `Into<Op>` (automatically generated by `effect!`)
/// - Type annotations on the result are recommended for clarity
/// - Multiple `perform!` calls can be chained naturally in sequence
#[proc_macro]
pub fn perform(ts: TokenStream) -> TokenStream {
    let input: syn::Expr = syn::parse(ts).unwrap();
    quote! {{
        let __eff = algae::Effect::new((#input).into());
        let __reply_opt = yield __eff;
        __reply_opt.unwrap().take::<_>()
    }}
    .into()
}