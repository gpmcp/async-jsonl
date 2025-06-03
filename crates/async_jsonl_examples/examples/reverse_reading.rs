//! Example demonstrating reverse reading of JSONL files
//!
//! This example shows how to use the .get_rev_n() method to read JSONL files
//! from end to beginning, similar to the Unix `tail` command.

use async_jsonl::Jsonl;
use futures::StreamExt;
use std::io::Cursor;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample JSONL data
    let sample_data = r#"{"timestamp": "2024-01-01T10:00:00Z", "level": "INFO", "message": "Application started"}
{"timestamp": "2024-01-01T10:01:00Z", "level": "DEBUG", "message": "Processing request 1"}
{"timestamp": "2024-01-01T10:02:00Z", "level": "WARN", "message": "High memory usage detected"}
{"timestamp": "2024-01-01T10:03:00Z", "level": "ERROR", "message": "Database connection failed"}
{"timestamp": "2024-01-01T10:04:00Z", "level": "INFO", "message": "Retrying connection"}
"#;

    println!("=== Example 1: Reading from memory (Cursor) ===");

    // Example 1: Reading from a Cursor (in-memory data)
    let cursor = Cursor::new(sample_data.as_bytes());
    let jsonl = Jsonl::new(cursor);

    println!("Reading last 5 lines in reverse order:");
    let rev_jsonl = jsonl.get_rev_n(5).await?;

    // Collect results and iterate
    let results: Vec<_> = rev_jsonl.collect().await;
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(line) => println!("{}: {}", i + 1, line),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    println!();

    // Example 2: Reading from a file
    println!("=== Example 2: Reading from file ===");

    let temp_file_path = "/tmp/example_log.jsonl";

    // Create a temporary file
    let mut file = File::create(temp_file_path).await?;
    file.write_all(sample_data.as_bytes()).await?;
    file.shutdown().await?;

    // Read the file in reverse
    let jsonl_file = Jsonl::from_path(temp_file_path).await?;
    let rev_jsonl_file = jsonl_file.get_rev_n(3).await?;

    println!("Reading file in reverse order (last 3 lines):");
    let results: Vec<_> = rev_jsonl_file.collect().await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(line) => println!("{}: {}", i + 1, line),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Clean up
    tokio::fs::remove_file(temp_file_path).await.ok();

    println!();

    // Example 3: Compare forward vs reverse reading
    println!("=== Example 3: Forward vs Reverse comparison ===");

    let cursor_forward = Cursor::new(sample_data.as_bytes());
    let jsonl_forward = Jsonl::new(cursor_forward);

    let cursor_reverse = Cursor::new(sample_data.as_bytes());
    let jsonl_reverse = Jsonl::new(cursor_reverse);

    println!("Forward reading:");
    let forward_results: Vec<_> = jsonl_forward.collect().await;
    for (i, result) in forward_results.iter().enumerate() {
        if let Ok(line) = result {
            println!("{}: {}", i + 1, line);
        }
    }

    println!("\nReverse reading (last 5 lines):");
    let rev_jsonl_compare = jsonl_reverse.get_rev_n(5).await?;
    let reverse_results: Vec<_> = rev_jsonl_compare.collect().await;
    for (i, result) in reverse_results.iter().enumerate() {
        if let Ok(line) = result {
            println!("{}: {}", i + 1, line);
        }
    }

    Ok(())
}
