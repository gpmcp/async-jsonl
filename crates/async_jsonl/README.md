# async-jsonl

An efficient async Rust library for reading and processing JSON Lines (JSONL) files using Tokio streams.

## Quick Start

### Reading Raw JSON Lines

```rust
use async_jsonl::Jsonl;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read from file
    let jsonl = Jsonl::from_path("data.jsonl").await?;
    
    // Process each line
    let lines: Vec<_> = jsonl.collect().await;
    for line_result in lines {
        let line = line_result?;
        println!("Raw JSON: {}", line);
    }
    
    Ok(())
}
```

### Deserializing to Structs

```rust
use async_jsonl::{Jsonl, JsonlDeserialize};
use futures::StreamExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: u32,
    city: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let jsonl = Jsonl::from_path("people.jsonl").await?;
    let people = jsonl.deserialize::<Person>();
    
    let results: Vec<_> = people.collect().await;
    for person_result in results {
        let person = person_result?;
        println!("{:?}", person);
    }
    
    Ok(())
}
```

### Streaming Processing

```rust
use async_jsonl::{Jsonl, JsonlDeserialize};
use futures::StreamExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let jsonl = Jsonl::from_path("records.jsonl").await?;
    let mut stream = jsonl.deserialize::<Record>();
    
    // Process one item at a time
    while let Some(record_result) = stream.next().await {
        let record = record_result?;
        
        // Filter and process on the fly
        if record.id % 2 == 0 {
            println!("Even ID: {:?}", record);
        }
    }
    
    Ok(())
}
```

### Working with serde_json::Value

```rust
use async_jsonl::{Jsonl, JsonlValueDeserialize, jsonl_values};
use futures::StreamExt;
use serde_json::Value;
use std::io::Cursor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}"#;
    let reader = Cursor::new(data.as_bytes());
    
    let jsonl = Jsonl::new(reader);
    let values = jsonl.deserialize_values();
    
    let results: Vec<Value> = values
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    
    for value in results {
        println!("Name: {}, Age: {}", value["name"], value["age"]);
    }
    
    Ok(())
}
```

### Reading from Memory

```rust
use async_jsonl::Jsonl;
use futures::StreamExt;
use std::io::Cursor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let jsonl_data = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}"#;
    
    let reader = Cursor::new(jsonl_data.as_bytes());
    let jsonl = Jsonl::new(reader);
    
    let lines: Vec<_> = jsonl.collect().await;
    println!("Read {} lines from memory", lines.len());
    
    Ok(())
}
```

### Counting Lines

```rust
use async_jsonl::{Jsonl,JsonlReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Count lines efficiently without full deserialization
    let jsonl = Jsonl::from_path("large_file.jsonl").await?;
    let count = jsonl.count().await?;
    
    println!("File contains {} non-empty lines", count);
    
    Ok(())
}
```

## Features

- **Async/Await**: Built on Tokio for efficient async I/O
- **Memory Efficient**: Stream-based processing, no need to load entire files
- **Type Safe**: Full serde integration for type-safe deserialization
- **Error Resilient**: Continue processing even when individual lines fail
- **Flexible Input**: Works with files, memory, or any `AsyncRead` source
