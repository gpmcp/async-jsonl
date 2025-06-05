//! Demonstration of RevBufReader streaming interface
//!
//! This example shows how to use the RevBufReader.lines() method to stream
//! lines in reverse order, similar to `tail -f` but reading from end to beginning.

use async_rev_buf::RevBufReader;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data
    let sample_data = b"First line\nSecond line\nThird line\nFourth line\nFifth line\n";

    println!("=== Streaming Lines in Reverse Order ===");

    // Create a RevBufReader from the data
    let cursor = Cursor::new(sample_data);
    let rev_reader = RevBufReader::new(cursor);
    let mut lines = rev_reader.lines();

    // Stream lines one by one in reverse order
    println!("Reading lines from end to beginning:");
    let mut line_number = 1;
    while let Some(line) = lines.next_line().await? {
        println!("{}: {}", line_number, line);
        line_number += 1;
    }

    Ok(())
}
