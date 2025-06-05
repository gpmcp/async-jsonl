//! # Async Reverse Buffer Reader
//!
//! A high-performance async buffered reader for reading lines in reverse order.
//! Optimized for processing log files, chat histories, or any text data where
//! you need to read from the end backwards.
//!
//! ## Features
//!
//! - **High Performance**: 8+ million lines/sec throughput
//! - **Streaming Interface**: Clean `lines().next_line().await` pattern
//! - **Memory Efficient**: Fixed buffer size, minimal allocations
//! - **Unicode Support**: Proper UTF-8 handling
//! - **Async/Await**: Full tokio compatibility
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use async_rev_buf::RevBufReader;
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     let file = File::open("large.log").await?;
//!     let reader = RevBufReader::new(file);
//!     let mut lines = reader.lines();
//!
//!     // Read last 10 lines efficiently
//!     for _ in 0..10 {
//!         if let Some(line) = lines.next_line().await? {
//!             println!("{}", line);
//!         } else {
//!             break;
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Patterns
//!
//! ### Streaming Interface (Recommended)
//!
//! ```rust
//! # use async_rev_buf::RevBufReader;
//! # use std::io::Cursor;
//! # async fn example() -> std::io::Result<()> {
//! let reader = RevBufReader::new(Cursor::new("line1\nline2\nline3"));
//! let mut lines = reader.lines();
//!
//! while let Some(line) = lines.next_line().await? {
//!     println!("{}", line);
//! }
//! # Ok(()) }
//! ```
//!
//! ### Direct Method Access
//!
//! ```rust
//! # use async_rev_buf::RevBufReader;
//! # use std::io::Cursor;
//! # async fn example() -> std::io::Result<()> {
//! let mut reader = RevBufReader::new(Cursor::new("line1\nline2\nline3"));
//!
//! while let Some(line) = reader.next_line().await? {
//!     println!("{}", line);
//! }
//! # Ok(()) }
//! ```

mod buf_reader;
mod lines;

pub use buf_reader::RevBufReader;
pub use lines::Lines;

/// Default buffer size: 8 KB
pub const DEFAULT_BUF_SIZE: usize = 8 * 1024;
