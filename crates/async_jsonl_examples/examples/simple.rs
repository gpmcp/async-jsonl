use async_jsonl::{Jsonl, JsonlDeserialize};
use futures::StreamExt;
use serde::Deserialize;
use std::io::Cursor;
use tokio;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u32,
    #[allow(dead_code)]
    city: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Sample JSONL data
    let jsonl_data = r#"{"name": "Alice", "age": 30, "city": "New York"}
{"name": "Bob", "age": 25, "city": "San Francisco"}
{"name": "Charlie", "age": 35, "city": "Chicago"}
"#;

    println!("=== Reading raw JSON lines ===");

    // Create a reader from the sample data
    let reader = Cursor::new(jsonl_data.as_bytes());
    let iterator = Jsonl::new(reader);

    // Collect and print raw JSON lines
    let raw_lines: Vec<_> = iterator.collect().await;
    for (i, line_result) in raw_lines.iter().enumerate() {
        match line_result {
            Ok(line) => println!("Line {}: {}", i + 1, line),
            Err(e) => eprintln!("Error reading line {}: {}", i + 1, e),
        }
    }

    println!("\n=== Deserializing to Person structs ===");

    // Create a new reader for deserialization
    let reader = Cursor::new(jsonl_data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<Person>();

    // Collect and print deserialized persons
    let persons: Vec<_> = deserializer.collect().await;
    for (i, person_result) in persons.iter().enumerate() {
        match person_result {
            Ok(person) => println!("Person {}: {:?}", i + 1, person),
            Err(e) => eprintln!("Error deserializing person {}: {}", i + 1, e),
        }
    }

    println!("\n=== Streaming processing ===");

    // Process items one by one using streaming
    let reader = Cursor::new(jsonl_data.as_bytes());
    let iterator = Jsonl::new(reader);
    let mut deserializer = iterator.deserialize::<Person>();

    let mut count = 0;
    while let Some(person_result) = deserializer.next().await {
        count += 1;
        match person_result {
            Ok(person) => {
                if person.age >= 30 {
                    println!("Found adult: {} (age {})", person.name, person.age);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Processed {} records total", count);

    Ok(())
}
