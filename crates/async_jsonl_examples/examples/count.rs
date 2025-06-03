//! Example demonstrating line counting capabilities

use async_jsonl::{Jsonl, JsonlDeserialize};
use futures::StreamExt;
use serde_json::Value;
use std::io::Cursor;
use tempfile::NamedTempFile;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example 1: Line counting from memory
    println!("=== Example 1: Line counting from memory ===");
    let data = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}

{"id": 3, "name": "Charlie"}

{"id": 4, "name": "Diana"}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);
    let count = jsonl.count_lines().await?;
    println!("Total non-empty lines: {}", count);

    // Example 2: Reading from file and counting
    println!("\n=== Example 2: File operations ===");
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();

    let file_data = r#"{"order": 1, "product": "Laptop", "price": 999.99}
{"order": 2, "product": "Mouse", "price": 29.99}
{"order": 3, "product": "Keyboard", "price": 79.99}
{"order": 4, "product": "Monitor", "price": 299.99}
{"order": 5, "product": "Webcam", "price": 89.99}
"#;

    fs::write(temp_path, file_data).await?;

    // Count lines in the file
    let file_line_count = Jsonl::from_path(temp_path).await?.count_lines().await?;
    println!("Lines in file: {}", file_line_count);

    // Example 3: Forward reading
    println!("\n=== Example 3: Forward reading ===");
    let jsonl = Jsonl::from_path(temp_path).await?;
    let mut forward_stream = jsonl.deserialize::<Value>();

    println!("Forward order:");
    while let Some(result) = forward_stream.next().await {
        let value = result?;
        println!(
            "  Order {}: {} - ${}",
            value["order"], value["product"], value["price"]
        );
    }

    // Example 5: Comparing counts
    println!("\n=== Example 5: Count comparison ===");
    let jsonl_for_count = Jsonl::from_path(temp_path).await?;
    let manual_count = jsonl_for_count.collect::<Vec<_>>().await.len();

    println!("File line count: {}", file_line_count);
    println!("Manual count: {}", manual_count);
    println!("Counts match: {}", file_line_count == manual_count);

    Ok(())
}
