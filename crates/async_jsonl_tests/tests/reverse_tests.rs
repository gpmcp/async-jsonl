use async_jsonl::{Jsonl, JsonlReader};
use futures::StreamExt;
use std::io::Cursor;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_get_rev_n_with_cursor() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
"#;

    let cursor = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(cursor);

    let rev_jsonl = jsonl
        .last_n(3)
        .await
        .expect("Failed to create reverse iterator");

    let results: Vec<_> = rev_jsonl.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let lines: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();

    // Should be in reverse order (last lines first)
    assert_eq!(lines[0], r#"{"id": 3, "name": "Charlie", "active": true}"#);
    assert_eq!(lines[1], r#"{"id": 2, "name": "Bob", "active": false}"#);
    assert_eq!(lines[2], r#"{"id": 1, "name": "Alice", "active": true}"#);
}

#[tokio::test]
async fn test_get_rev_n_with_file() {
    let temp_file_path = "/tmp/test_reverse_jsonl.jsonl";

    // Create a test file
    let mut file = File::create(temp_file_path)
        .await
        .expect("Failed to create temp file");
    let data = r#"{"line": 1}
{"line": 2}
{"line": 3}
{"line": 4}
{"line": 5}
"#;
    file.write_all(data.as_bytes())
        .await
        .expect("Failed to write to temp file");
    file.shutdown().await.expect("Failed to close file");

    // Test getting last 3 lines
    let jsonl = Jsonl::from_path(temp_file_path)
        .await
        .expect("Failed to open file");
    let rev_jsonl = jsonl
        .last_n(3)
        .await
        .expect("Failed to create reverse iterator");

    let results: Vec<_> = rev_jsonl.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let lines: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();

    // Should be last 3 lines in reverse order
    assert_eq!(lines[0], r#"{"line": 5}"#);
    assert_eq!(lines[1], r#"{"line": 4}"#);
    assert_eq!(lines[2], r#"{"line": 3}"#);

    // Clean up
    tokio::fs::remove_file(temp_file_path).await.ok();
}

#[tokio::test]
async fn test_get_rev_n_empty_file() {
    let cursor = Cursor::new(b"");
    let jsonl = Jsonl::new(cursor);

    let rev_jsonl = jsonl
        .last_n(5)
        .await
        .expect("Failed to create reverse iterator");
    let results: Vec<_> = rev_jsonl.collect().await;

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_get_rev_n_single_line() {
    let data = r#"{"single": "line"}
"#;

    let cursor = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(cursor);

    let rev_jsonl = jsonl
        .last_n(1)
        .await
        .expect("Failed to create reverse iterator");
    let results: Vec<_> = rev_jsonl.collect().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    let line = results.into_iter().next().unwrap().unwrap();
    assert_eq!(line, r#"{"single": "line"}"#);
}

#[tokio::test]
async fn test_get_rev_n_with_empty_lines() {
    let data = r#"{"id": 1}

{"id": 2}


{"id": 3}

"#;

    let cursor = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(cursor);

    let rev_jsonl = jsonl
        .last_n(3)
        .await
        .expect("Failed to create reverse iterator");
    let results: Vec<_> = rev_jsonl.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let lines: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();

    // Should be in reverse order, with empty lines filtered out
    assert_eq!(lines[0], r#"{"id": 3}"#);
    assert_eq!(lines[1], r#"{"id": 2}"#);
    assert_eq!(lines[2], r#"{"id": 1}"#);
}

#[tokio::test]
async fn test_get_n_lines() {
    let data = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}
{"name": "Dave", "age": 40}
{"name": "Eve", "age": 28}
"#;

    let cursor = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(cursor);

    let first_3 = jsonl.first_n(3).await.unwrap();
    let results: Vec<_> = first_3.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let lines: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();

    // Should be first 3 lines in order
    assert_eq!(lines[0], r#"{"name": "Alice", "age": 30}"#);
    assert_eq!(lines[1], r#"{"name": "Bob", "age": 25}"#);
    assert_eq!(lines[2], r#"{"name": "Charlie", "age": 35}"#);
}

#[tokio::test]
async fn test_compare_forward_and_reverse() {
    let data = r#"{"test": 1}
{"test": 2}
{"test": 3}
"#;

    // Forward reading first 3
    let cursor_forward = Cursor::new(data.as_bytes());
    let jsonl_forward = Jsonl::new(cursor_forward);
    let forward_stream = jsonl_forward.first_n(3).await.unwrap();
    let forward_results: Vec<_> = forward_stream.collect().await;
    let forward_lines: Vec<String> = forward_results.into_iter().map(|r| r.unwrap()).collect();

    // Reverse reading last 3
    let cursor_reverse = Cursor::new(data.as_bytes());
    let jsonl_reverse = Jsonl::new(cursor_reverse);
    let rev_jsonl = jsonl_reverse
        .last_n(3)
        .await
        .expect("Failed to create reverse iterator");
    let reverse_results: Vec<_> = rev_jsonl.collect().await;
    let reverse_lines: Vec<String> = reverse_results.into_iter().map(|r| r.unwrap()).collect();

    // They should be exactly opposite
    assert_eq!(forward_lines.len(), reverse_lines.len());

    let mut expected_reverse = forward_lines.clone();
    expected_reverse.reverse();

    assert_eq!(reverse_lines, expected_reverse);
}
