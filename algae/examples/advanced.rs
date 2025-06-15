#![feature(coroutines, coroutine_trait, yield_expr)]
use algae::prelude::*;
use std::collections::HashMap;

// Define multiple effect families as shown in README
effect! {
    // File operations
    FileSystem::Read (String) -> Result<String, std::io::Error>;
    FileSystem::Write ((String, String)) -> Result<(), std::io::Error>;

    // Database operations (simplified for demo)
    Database::Query (String) -> Vec<String>;
    Database::Execute (String) -> Result<u64, String>;

    // Logging operations
    Logger::Info (String) -> ();
    Logger::Error (String) -> ();
}

// A simple Row type for database results (used conceptually in comments)
#[allow(dead_code)]
type Row = String;

// Complex effectful function as shown in README
#[effectful]
fn process_file(filename: String) -> Result<usize, String> {
    let _: () = perform!(Logger::Info(format!("Processing {filename}")));

    let content: Result<String, std::io::Error> = perform!(FileSystem::Read(filename.clone()));
    let content = content.map_err(|e| format!("Read error: {e}"))?;

    let rows: Vec<String> = perform!(Database::Query(format!(
        "INSERT INTO files (name, content) VALUES ('{}', '{}')",
        filename, content
    )));

    let row_count = rows.len();
    let _: () = perform!(Logger::Info(format!("Inserted {row_count} rows")));
    Ok(rows.len())
}

// Production-like handler (simplified for demo)
struct ProductionHandler {
    files: HashMap<String, String>, // Simulated file system
    logs: Vec<String>,              // Captured logs
}

impl ProductionHandler {
    fn new() -> Self {
        let mut files = HashMap::new();
        files.insert(
            "test.txt".to_string(),
            "Hello, World!\nThis is a test file.".to_string(),
        );
        files.insert("data.txt".to_string(), "Line 1\nLine 2\nLine 3".to_string());

        Self {
            files,
            logs: Vec::new(),
        }
    }
}

impl Handler<Op> for ProductionHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileSystem(FileSystem::Read(path)) => {
                if let Some(content) = self.files.get(path) {
                    Box::new(Ok::<String, std::io::Error>(content.clone()))
                } else {
                    Box::new(Err::<String, std::io::Error>(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    )))
                }
            }
            Op::FileSystem(FileSystem::Write((path, content))) => {
                self.files.insert(path.clone(), content.clone());
                Box::new(Ok::<(), std::io::Error>(()))
            }
            Op::Database(Database::Query(sql)) => {
                // Simulate database insert returning affected rows
                let rows = vec![
                    format!("Row 1 for query: {sql}"),
                    format!("Row 2 for query: {sql}"),
                ];
                Box::new(rows)
            }
            Op::Database(Database::Execute(_sql)) => {
                // Simulate database execute returning count
                Box::new(Ok::<u64, String>(2))
            }
            Op::Logger(Logger::Info(msg)) => {
                println!("[INFO] {msg}");
                self.logs.push(format!("INFO: {msg}"));
                Box::new(())
            }
            Op::Logger(Logger::Error(msg)) => {
                println!("[ERROR] {msg}");
                self.logs.push(format!("ERROR: {msg}"));
                Box::new(())
            }
        }
    }
}

// Mock handler for testing as shown in README
struct MockHandler {
    files: HashMap<String, String>,
    logs: Vec<String>,
    db_responses: Vec<Vec<String>>,
    db_index: std::cell::RefCell<usize>,
}

impl MockHandler {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
            logs: Vec::new(),
            db_responses: vec![vec!["Mock row 1".to_string(), "Mock row 2".to_string()]],
            db_index: std::cell::RefCell::new(0),
        }
    }

    fn with_file(mut self, path: String, content: String) -> Self {
        self.files.insert(path, content);
        self
    }
}

impl Handler<Op> for MockHandler {
    fn handle(&mut self, op: &Op) -> Box<dyn std::any::Any + Send> {
        match op {
            Op::FileSystem(FileSystem::Read(path)) => {
                Box::new(self.files.get(path).cloned().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")
                }))
            }
            Op::FileSystem(FileSystem::Write((path, content))) => {
                self.files.insert(path.clone(), content.clone());
                Box::new(Ok::<(), std::io::Error>(()))
            }
            Op::Database(Database::Query(_sql)) => {
                let mut index = self.db_index.borrow_mut();
                let response = if *index < self.db_responses.len() {
                    self.db_responses[*index].clone()
                } else {
                    vec!["Default mock response".to_string()]
                };
                *index += 1;
                Box::new(response)
            }
            Op::Database(Database::Execute(_sql)) => Box::new(Ok::<u64, String>(1)),
            Op::Logger(Logger::Info(msg)) => {
                self.logs.push(format!("MOCK INFO: {msg}"));
                Box::new(())
            }
            Op::Logger(Logger::Error(msg)) => {
                self.logs.push(format!("MOCK ERROR: {msg}"));
                Box::new(())
            }
        }
    }
}

// Additional effectful functions demonstrating composition
#[effectful]
fn batch_process(filenames: Vec<String>) -> Vec<Result<usize, String>> {
    let _: () = perform!(Logger::Info("Starting batch processing".to_string()));

    let mut results = Vec::new();
    for filename in filenames {
        let result = process_file(filename)
            .handle(ProductionHandler::new())
            .run();
        results.push(result);
    }

    let file_count = results.len();
    let _: () = perform!(Logger::Info(format!("Processed {file_count} files")));
    results
}

#[effectful]
fn create_report() -> String {
    let _: () = perform!(Logger::Info("Generating report".to_string()));

    let data: Vec<String> = perform!(Database::Query("SELECT * FROM processed_files".to_string()));
    let record_count = data.len();
    let report = format!("Report: Found {record_count} records");

    let _: Result<(), std::io::Error> = perform!(FileSystem::Write((
        "report.txt".to_string(),
        report.clone()
    )));

    let _: () = perform!(Logger::Info("Report generated successfully".to_string()));
    report
}

fn main() {
    println!("=== Advanced Algae Example ===\n");

    // Example 1: Single file processing with production handler
    println!("1. Processing single file:");
    let result = process_file("test.txt".to_string())
        .handle(ProductionHandler::new())
        .run();

    match result {
        Ok(count) => println!("   Successfully processed file, inserted {count} rows"),
        Err(e) => println!("   Error: {e}"),
    }

    // Example 2: Testing with mock handler
    println!("\n2. Testing with mock handler:");
    let mock_handler =
        MockHandler::new().with_file("test.txt".to_string(), "mock file content".to_string());

    let mock_result = process_file("test.txt".to_string())
        .handle(mock_handler)
        .run();

    match mock_result {
        Ok(count) => println!("   Mock test passed, processed {count} rows"),
        Err(e) => println!("   Mock test failed: {e}"),
    }

    // Example 3: Report generation
    println!("\n3. Generating report:");
    let report = create_report().handle(ProductionHandler::new()).run();
    println!("   {report}");

    // Example 4: Error handling
    println!("\n4. Error handling (non-existent file):");
    let error_result = process_file("nonexistent.txt".to_string())
        .handle(ProductionHandler::new())
        .run();

    match error_result {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   Expected error: {e}"),
    }

    println!("\n=== Example completed successfully! ===");
}

// Unit test demonstrating the testing pattern from README
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_file() {
        let mut handler = MockHandler::new();
        handler
            .files
            .insert("test.txt".to_string(), "test content".to_string());

        let result = process_file("test.txt".to_string()).handle(handler).run();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // Mock returns 2 rows
    }

    #[test]
    fn test_file_not_found() {
        let handler = MockHandler::new();

        let result = process_file("missing.txt".to_string())
            .handle(handler)
            .run();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Read error"));
    }
}
