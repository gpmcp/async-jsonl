# async-rev-buf

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A high-performance async buffered reader for reading lines in reverse order from files and streams.

## Overview

`async-rev-buf` provides `RevBufReader`, an async buffered reader that reads lines from the end of a file or stream backwards to the beginning. This is particularly useful for processing log files, JSON Lines data, or any line-oriented data where you need the most recent entries first.

## Features

- **Reverse Line Reading**: Read lines from end to beginning efficiently
- **Async/Await Support**: Full compatibility with tokio's async ecosystem
- **Configurable Buffering**: Customizable buffer sizes for optimal performance
- **Memory Efficient**: Chunked reading approach minimizes memory usage
- **Unicode Support**: Proper handling of UTF-8 text and various line endings
- **Streaming Interface**: Iterator-like API following tokio patterns

## Quick Start

```rust
use async_rev_buf::RevBufReader;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a file
    let file = File::open("data.txt").await?;
    
    // Create a reverse buffered reader
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();
    
    // Read lines in reverse order (last line first)
    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
    }
    
    Ok(())
}
```

## Use Cases

### Log File Processing

```rust
// Read the last 10 entries from a log file
let file = File::open("app.log").await?;
let reader = RevBufReader::new(file);
let mut lines = reader.lines();
let mut count = 0;

while let Some(line) = lines.next_line().await? {
    println!("{}", line);
    count += 1;
    if count >= 10 {
        break;
    }
}
```

### JSON Lines Processing

```rust
// Process JSONL data in reverse chronological order
let file = File::open("events.jsonl").await?;
let reader = RevBufReader::new(file);
let mut lines = reader.lines();

while let Some(line) = lines.next_line().await? {
    if let Ok(event) = serde_json::from_str::<Event>(&line) {
        println!("Event: {:?}", event);
    }
}
```

### Custom Buffer Size

```rust
// Use a larger buffer for better performance with large files
let file = File::open("large_file.txt").await?;
let reader = RevBufReader::with_capacity(64 * 1024, file); // 64KB buffer
let mut lines = reader.lines();
```

## API Reference

### RevBufReader

#### Constructor Methods
- `RevBufReader::new(reader)` - Create with default buffer size (8KB)
- `RevBufReader::with_capacity(capacity, reader)` - Create with custom buffer size

#### Reader Requirements
The inner reader must implement `AsyncRead + AsyncSeek + Unpin`.

#### Methods
- `lines()` - Returns a `Lines` iterator for streaming line access
- `get_ref()` - Get reference to underlying reader
- `get_mut()` - Get mutable reference to underlying reader
- `into_inner()` - Consume and return underlying reader
- `buffer()` - Access internal buffer contents

### Lines Iterator

- `next_line().await` - Get the next line (reading backwards)
- `get_ref()` - Get reference to underlying `RevBufReader`
- `get_mut()` - Get mutable reference to underlying `RevBufReader`
- `into_inner()` - Consume and return underlying `RevBufReader`

## Performance

`RevBufReader` is designed for efficiency:

- **Chunked Reading**: Reads data in chunks backwards, minimizing I/O operations
- **Configurable Buffers**: Tune buffer size based on your use case
- **Memory Efficient**: Only keeps necessary data in memory
- **Line Boundary Detection**: Efficiently detects line boundaries while reading backwards

*Times are approximate and depend on hardware and file characteristics.*

## Limitations

- Requires `AsyncSeek` - works with files but not all streams
- Reads entire lines into memory (suitable for typical text files)
- Line detection based on `\n` boundaries (handles `\r\n` correctly)

## Examples

See the `examples/` directory for complete usage examples:

```bash
cargo run --example streaming_demo
```

## Contributing

Contributions welcome! Please feel free to submit issues, feature requests, or pull requests.

*Part of the async-jsonl ecosystem for efficient async JSON Lines processing.*
