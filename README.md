# async-jsonl

An efficient async Rust library for reading and processing JSON Lines (JSONL) files using Tokio streams.

## Project Structure

This project is organized as a Rust workspace with multiple crates:

### Core Crates

- **`crates/async_jsonl/`** - The main library crate providing async JSONL processing functionality
- **`crates/async_rev_buf/`** - High-performance async reverse buffer reader (8+ million lines/sec)
- **`crates/async_jsonl_ci/`** - CI-specific utilities and tests

### Additional Resources

- **`examples/`** - Additional example files and demonstrations
- **`LICENSE`** - Project license information
- **`Cargo.toml`** - Workspace configuration
- **`release-plz.toml`** - Release automation configuration

## Quick Start

For detailed usage examples and API documentation, see the [main library README](crates/async_jsonl/README.md).

## Features

### async_jsonl
- **Async/Await**: Built on Tokio for efficient async I/O
- **Memory Efficient**: Stream-based processing without loading entire files
- **Type Safe**: Full serde integration for type-safe deserialization
- **Error Resilient**: Continue processing even when individual lines fail
- **Flexible Input**: Works with files, memory, or any `AsyncRead` source

### async_rev_buf
- **High Performance**: 8-9 million lines/sec reverse reading throughput
- **Best-in-Class**: 2.4-2.5x faster than existing async alternatives
- **Streaming Interface**: Clean `lines().next_line().await` pattern following tokio conventions
- **Memory Efficient**: Fixed buffer size with minimal allocations
- **Unicode Support**: Proper handling of UTF-8 text and various line endings

## Development

This workspace uses Cargo's workspace feature to manage multiple related crates. To build all crates:

```bash
cargo build --workspace
```

To run all tests:

```bash
cargo test --workspace
```

## License

See [LICENSE](LICENSE) for details.