//! Basic integration tests for async_rev_buf crate

use async_rev_buf::RevBufReader;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::fs::File;

#[tokio::test]
async fn test_basic_reverse_reading() {
    // Create a temporary file with test content
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = "first line\nsecond line\nthird line\nfourth line";
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Read lines in reverse
    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(
        result,
        vec!["fourth line", "third line", "second line", "first line"]
    );
}

#[tokio::test]
async fn test_empty_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let line = lines.next_line().await.unwrap();
    assert!(line.is_none());
}

#[tokio::test]
async fn test_single_line() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"only line").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let line1 = lines.next_line().await.unwrap();
    assert_eq!(line1, Some("only line".to_string()));

    let line2 = lines.next_line().await.unwrap();
    assert!(line2.is_none());
}

#[tokio::test]
async fn test_custom_buffer_size() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"line1\nline2\nline3").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::with_capacity(8, file); // Small buffer
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(result, vec!["line3", "line2", "line1"]);
}

#[tokio::test]
async fn test_unicode_content() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let content = "🎉 first line\n한국어 second line\nрусский third line";
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(
        result,
        vec!["русский third line", "한국어 second line", "🎉 first line"]
    );
}

#[tokio::test]
async fn test_accessor_methods() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let mut reader = RevBufReader::new(file);

    // Test accessor methods
    assert!(reader.get_ref().metadata().await.is_ok());
    assert!(reader.get_mut().metadata().await.is_ok());

    // Test buffer method
    assert!(reader.buffer().is_empty());

    // Test into_inner
    let _inner = reader.into_inner();
}

#[tokio::test]
async fn test_lines_methods() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"line1\nline2").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    // Test accessor methods
    let _reader_ref = lines.get_ref();
    let _reader_mut = lines.get_mut();

    // Read one line
    let line = lines.next_line().await.unwrap();
    assert_eq!(line, Some("line2".to_string()));

    // Test into_inner
    let _reader = lines.into_inner();
}
