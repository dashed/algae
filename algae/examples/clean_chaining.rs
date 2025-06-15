//! Example demonstrating the cleanest handler chaining syntax.
//!
//! This example shows the simplest way to chain partial handlers
//! using `.begin_chain().handle().handle().handle()`.

#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;

// Define our effects
effect! {
    Auth::Validate (String) -> Result<String, String>;
    Database::Query (String) -> Vec<String>;
    Cache::Get (String) -> Option<String>;
    Cache::Set ((String, String)) -> ();
}

// Authentication handler
struct AuthHandler;

impl PartialHandler<Op> for AuthHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Auth(Auth::Validate(token)) => {
                if token == "valid-token" {
                    Some(Box::new(Ok::<String, String>("user123".to_string())))
                } else {
                    Some(Box::new(Err::<String, String>("Invalid token".to_string())))
                }
            }
            _ => None,
        }
    }
}

// Database handler
struct DatabaseHandler;

impl PartialHandler<Op> for DatabaseHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Database(Database::Query(user_id)) => {
                if user_id == "user123" {
                    Some(Box::new(vec![
                        "post1".to_string(),
                        "post2".to_string(),
                        "post3".to_string(),
                    ]))
                } else {
                    Some(Box::new(Vec::<String>::new()))
                }
            }
            _ => None,
        }
    }
}

// Cache handler
struct CacheHandler {
    cache: std::collections::HashMap<String, String>,
}

impl CacheHandler {
    fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }
}

impl PartialHandler<Op> for CacheHandler {
    fn maybe_handle(&mut self, op: &Op) -> Option<Box<dyn std::any::Any + Send>> {
        match op {
            Op::Cache(Cache::Get(key)) => Some(Box::new(self.cache.get(key).cloned())),
            Op::Cache(Cache::Set((key, value))) => {
                self.cache.insert(key.clone(), value.clone());
                Some(Box::new(()))
            }
            _ => None,
        }
    }
}

// Application logic
#[effectful]
fn get_user_posts(token: String) -> Result<Vec<String>, String> {
    // Check cache first
    let cache_key = format!("posts:{token}");
    let cached: Option<String> = perform!(Cache::Get(cache_key.clone()));

    if let Some(cached_data) = cached {
        println!("Cache hit!");
        return Ok(vec![cached_data]);
    }

    println!("Cache miss, validating token...");

    // Validate token
    let user_result: Result<String, String> = perform!(Auth::Validate(token));
    let user_id = user_result?;

    // Query database
    let posts: Vec<String> = perform!(Database::Query(user_id));

    // Cache the result
    if !posts.is_empty() {
        let _: () = perform!(Cache::Set((cache_key, posts.join(","))));
    }

    Ok(posts)
}

fn main() {
    println!("=== Clean Handler Chaining Demo ===\n");

    // The cleanest syntax: begin_chain() followed by handle() calls
    let result = get_user_posts("valid-token".to_string())
        .begin_chain()
        .handle(AuthHandler)
        .handle(DatabaseHandler)
        .handle(CacheHandler::new())
        .run_checked();

    match result {
        Ok(Ok(posts)) => {
            println!("\nUser posts:");
            for post in posts {
                println!("  - {post}");
            }
        }
        Ok(Err(err)) => println!("\nAuthentication error: {err}"),
        Err(UnhandledOp(op)) => eprintln!("\nUnhandled operation: {op:?}"),
    }

    // Second call should hit the cache
    println!("\n=== Second call (should hit cache) ===\n");

    let result = get_user_posts("valid-token".to_string())
        .begin_chain()
        .handle(AuthHandler)
        .handle(DatabaseHandler)
        .handle(CacheHandler::new()) // Note: new cache instance, so won't hit
        .run_checked();

    match result {
        Ok(Ok(_)) => println!("Retrieved posts"),
        Ok(Err(err)) => println!("Error: {err}"),
        Err(UnhandledOp(op)) => eprintln!("Unhandled: {op:?}"),
    }

    // Example with invalid token
    println!("\n=== Invalid token example ===\n");

    let result = get_user_posts("invalid-token".to_string())
        .begin_chain()
        .handle(AuthHandler)
        .handle(DatabaseHandler)
        .handle(CacheHandler::new())
        .run_checked();

    match result {
        Ok(Ok(_)) => println!("This shouldn't happen"),
        Ok(Err(err)) => println!("Expected error: {err}"),
        Err(UnhandledOp(op)) => eprintln!("Unhandled: {op:?}"),
    }
}
