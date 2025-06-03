# async-jsonl

An efficient async Rust library for reading and processing JSON Lines (JSONL) files using Tokio streams.

## Project Structure

This project is organized as a Rust workspace with multiple crates:

### Core Crates

- **`crates/async_jsonl/`** - The main library crate providing async JSONL processing functionality
- **`crates/async_jsonl_examples/`** - Example code demonstrating library usage
- **`crates/async_jsonl_tests/`** - Integration tests for the library
- **`crates/async_jsonl_ci/`** - CI-specific utilities and tests

### Additional Resources

- **`examples/`** - Additional example files and demonstrations
- **`LICENSE`** - Project license information
- **`Cargo.toml`** - Workspace configuration
- **`release-plz.toml`** - Release automation configuration

## Quick Start

For detailed usage examples and API documentation, see the [main library README](crates/async_jsonl/README.md).

## Features

- **Async/Await**: Built on Tokio for efficient async I/O
- **Memory Efficient**: Stream-based processing without loading entire files
- **Type Safe**: Full serde integration for type-safe deserialization
- **Error Resilient**: Continue processing even when individual lines fail
- **Flexible Input**: Works with files, memory, or any `AsyncRead` source

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