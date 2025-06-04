//! Example demonstrating how to count lines in JSONL files
//!
//! This example shows how to use the .count() method to efficiently count
//! the total number of lines in a JSONL file without loading all content into memory.

use async_jsonl::{Jsonl, JsonlReader};
use std::io::Cursor;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== JSONL Line Counting Examples ===");

    // Example 1: Count lines in memory data
    println!("\n=== Example 1: Counting lines in memory ===");

    let sample_data = r#"{"id": 1, "name": "Alice", "department": "Engineering"}
{"id": 2, "name": "Bob", "department": "Marketing"}
{"id": 3, "name": "Charlie", "department": "Sales"}
{"id": 4, "name": "Diana", "department": "HR"}
{"id": 5, "name": "Eve", "department": "Engineering"}
"#;

    let reader = Cursor::new(sample_data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let line_count = JsonlReader::count(jsonl).await;
    println!("Total lines in sample data: {}", line_count);

    // Example 2: Count lines in a file
    println!("\n=== Example 2: Counting lines in a file ===");

    let temp_file_path = "/tmp/employee_data.jsonl";

    // Create a sample file with employee data
    let mut file = File::create(temp_file_path).await?;
    let file_data = r#"{"id": 1, "name": "Alice", "salary": 75000, "active": true}
{"id": 2, "name": "Bob", "salary": 68000, "active": true}
{"id": 3, "name": "Charlie", "salary": 82000, "active": false}
{"id": 4, "name": "Diana", "salary": 71000, "active": true}
{"id": 5, "name": "Eve", "salary": 79000, "active": true}
{"id": 6, "name": "Frank", "salary": 65000, "active": false}
{"id": 7, "name": "Grace", "salary": 88000, "active": true}
"#;
    file.write_all(file_data.as_bytes()).await?;
    file.shutdown().await?;

    // Count lines in the file
    let jsonl_file = Jsonl::from_path(temp_file_path).await?;
    let file_line_count = JsonlReader::count(jsonl_file).await;
    println!(
        "Total lines in file '{}': {}",
        temp_file_path, file_line_count
    );

    // Example 3: Handling empty files and edge cases
    println!("\n=== Example 3: Edge cases ===");

    // Empty data
    let empty_reader = Cursor::new(b"");
    let empty_jsonl = Jsonl::new(empty_reader);
    let empty_count = JsonlReader::count(empty_jsonl).await;
    println!("Lines in empty data: {}", empty_count);

    // Single line
    let single_line = r#"{"single": "record"}"#;
    let single_reader = Cursor::new(single_line.as_bytes());
    let single_jsonl = Jsonl::new(single_reader);
    let single_count = JsonlReader::count(single_jsonl).await;
    println!("Lines in single-line data: {}", single_count);

    // Data with empty lines (they get filtered out)
    let data_with_empty_lines = r#"{"line": 1}

{"line": 2}


{"line": 3}

"#;
    let empty_lines_reader = Cursor::new(data_with_empty_lines.as_bytes());
    let empty_lines_jsonl = Jsonl::new(empty_lines_reader);
    let filtered_count = JsonlReader::count(empty_lines_jsonl).await;
    println!(
        "Lines in data with empty lines (filtered): {}",
        filtered_count
    );

    // Example 4: Performance demonstration with larger data
    println!("\n=== Example 4: Performance with larger dataset ===");

    // Generate a larger dataset
    let mut large_data = String::new();
    for i in 0..10000 {
        large_data.push_str(&format!(
            "{{\"id\": {}, \"timestamp\": \"2024-01-01T{:02}:{:02}:00Z\", \"value\": {}}}\n",
            i,
            i % 24, // hours (0-23)
            i % 60, // minutes (0-59)
            i * 42  // some computed value
        ));
    }

    let large_reader = Cursor::new(large_data.as_bytes());
    let large_jsonl = Jsonl::new(large_reader);

    let start = std::time::Instant::now();
    let large_count = JsonlReader::count(large_jsonl).await;
    let elapsed = start.elapsed();

    println!("Counted {} lines in {:?}", large_count, elapsed);
    println!(
        "Performance: {:.0} lines/sec",
        large_count as f64 / elapsed.as_secs_f64()
    );

    // Example 5: Practical use case - counting records by file extension
    println!("\n=== Example 5: Practical use case ===");

    let log_files = vec![
        (
            "/tmp/app_errors.jsonl",
            r#"{"level": "ERROR", "message": "Database timeout"}
{"level": "ERROR", "message": "Authentication failed"}
{"level": "ERROR", "message": "Rate limit exceeded"}
"#,
        ),
        (
            "/tmp/app_info.jsonl",
            r#"{"level": "INFO", "message": "Server started"}
{"level": "INFO", "message": "User logged in"}
{"level": "INFO", "message": "Request processed"}
{"level": "INFO", "message": "Cache refreshed"}
{"level": "INFO", "message": "Backup completed"}
"#,
        ),
        (
            "/tmp/app_debug.jsonl",
            r#"{"level": "DEBUG", "message": "Processing request ID 123"}
{"level": "DEBUG", "message": "Query executed in 45ms"}
"#,
        ),
    ];

    let mut total_log_entries = 0;

    for (file_path, content) in log_files {
        // Create the log file
        let mut log_file = File::create(file_path).await?;
        log_file.write_all(content.as_bytes()).await?;
        log_file.shutdown().await?;

        // Count entries in this log file
        let log_jsonl = Jsonl::from_path(file_path).await?;
        let log_count = JsonlReader::count(log_jsonl).await;
        total_log_entries += log_count;

        println!("Log file '{}': {} entries", file_path, log_count);

        // Clean up
        tokio::fs::remove_file(file_path).await.ok();
    }

    println!("Total log entries across all files: {}", total_log_entries);

    // Clean up the temp file
    tokio::fs::remove_file(temp_file_path).await.ok();

    println!("\n=== Summary ===");
    println!("The count() method provides an efficient way to:");
    println!("• Count total lines in JSONL files without loading all data");
    println!("• Handle large files with good performance");
    println!("• Filter out empty lines automatically");
    println!("• Work with both in-memory data and files");
    println!("• Process multiple files for aggregate statistics");

    Ok(())
}
