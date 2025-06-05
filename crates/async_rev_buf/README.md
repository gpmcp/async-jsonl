# async-rev-buf

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A high-performance async buffered reader for reading lines in reverse order from files and streams.

## Overview

`async-rev-buf` provides `RevBufReader`, an async buffered reader that reads lines from the end of a file or stream
backwards to the beginning. This is particularly useful for processing log files, JSON Lines data, or any line-oriented
data where you need the most recent entries first.

## Features

- **High Performance**: 8+ million lines/sec reverse reading throughput
- **Async/Await Support**: Full compatibility with tokio's async ecosystem
- **Streaming Interface**: Clean `lines().next_line().await` pattern following tokio conventions
- **Memory Efficient**: Fixed buffer size with minimal allocations
- **Unicode Support**: Proper handling of UTF-8 text and various line endings
- **Dual API**: Both streaming interface and direct method access
- **Type Safety**: Purpose-built API prevents forward/reverse reading confusion

## Quick Start

### Streaming Interface (Recommended)

```rust
use async_rev_buf::RevBufReader;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("large.log").await?;
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    // Read last 10 lines efficiently
    for _ in 0..10 {
        if let Some(line) = lines.next_line().await? {
            println!("{}", line);
        } else {
            break;
        }
    }

    Ok(())
}
```

### Direct Method Access

```rust
use async_rev_buf::RevBufReader;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data.txt").await?;
    let mut reader = RevBufReader::new(file);

    // Read lines in reverse order (last line first)
    while let Some(line) = reader.next_line().await? {
        println!("{}", line);
    }

    Ok(())
}
```

## Performance

### Latest Benchmark Results

Comprehensive performance comparison against all available async and sync reverse readers:

| Lines | **RevBufReader**     | **rev_buf_reader (sync, memory intensive)** | **tokio-rev-lines** | **Performance** |
|-------|----------------------|---------------------------------------------|---------------------|-----------------|
| 100   | **9.1M lines/sec**   | 12.8M lines/sec                             | 3.8M lines/sec      | **2.4x faster** |
| 1,000 | **8.5M lines/sec**   | 12.8M lines/sec                             | 3.5M lines/sec      | **2.4x faster** |
| 5,000 | **8.4M lines/sec**   | 13.1M lines/sec                             | 3.3M lines/sec      | **2.5x faster** |

### Performance Analysis

**🏆 Outstanding Async Performance:**

- **8-9 million lines/sec** consistently across all test sizes
- **71% of sync performance** while maintaining full async capabilities
- **2.4-2.5x faster** than existing async alternatives (tokio-rev-lines)
- **Scales well** with larger files

**🎯 When to Choose Our RevBufReader:**

- Building async/await applications
- Need concurrent file processing
- Integrating with tokio ecosystem
- Want non-blocking I/O
- Processing multiple files simultaneously
- Need the **fastest async reverse reader** available

**📊 Competitive Analysis:**

- **vs tokio-rev-lines**: 2.4-2.5x performance improvement
- **vs sync libraries**: 71% performance while staying async
- **Memory Efficient**: Fixed 8KB buffer (configurable)
- **Best-in-class**: Leading async reverse reading performance

### Run Benchmarks

```bash
cargo bench --bench comparison
```

## API Design Philosophy

**Purpose-Built for Reverse Reading:**

Instead of forcing compatibility with `AsyncBufRead` (which would cause 50-70% performance loss), we provide a **clean,
purpose-built API** optimized specifically for reverse reading.

**Benefits of Current Design:**

- 🚀 **Maximum Performance**: No compatibility overhead
- 🎯 **Clear Semantics**: Obviously designed for reverse reading
- 🔒 **Type Safety**: Prevents accidental forward/reverse confusion
- 📖 **Better UX**: Clear documentation and intent

## Limitations

**Technical Constraints:**

- Requires `AsyncSeek` - works with files but not all streams
- Designed for text files with reasonable line lengths
- Line detection based on `\n` and `\r\n` boundaries
- Not compatible with `AsyncBufRead` trait (by design for performance)

**Performance Context:**

- **Optimized for async**: 8+ million lines/sec is excellent for async reverse reading
- **Use case matters**: Perfect for log tailing, recent data access, concurrent processing

## Contributing

Contributions welcome! Please feel free to submit issues, feature requests, or pull requests.

**Focus Areas:**

- Performance optimizations
- Additional async stream types support
- Documentation improvements
- Real-world usage examples

*Part of the async-jsonl ecosystem for efficient async JSON Lines processing.*