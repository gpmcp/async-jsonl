use async_jsonl::{Jsonl, JsonlDeserialize, JsonlReader, JsonlValueDeserialize};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::io::Cursor;

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct TestRecord {
    id: u32,
    name: String,
    active: bool,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
struct SimpleRecord {
    value: i32,
}

#[tokio::test]
async fn test_basic_jsonl_reading() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);

    let results: Vec<_> = iterator.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let lines: Vec<String> = results.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(lines[0], r#"{"id": 1, "name": "Alice", "active": true}"#);
    assert_eq!(lines[1], r#"{"id": 2, "name": "Bob", "active": false}"#);
    assert_eq!(lines[2], r#"{"id": 3, "name": "Charlie", "active": true}"#);
}

#[tokio::test]
async fn test_jsonl_deserialization() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<TestRecord>();

    let results: Vec<_> = deserializer.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let records: Vec<TestRecord> = results.into_iter().map(|r| r.unwrap()).collect();

    assert_eq!(
        records[0],
        TestRecord {
            id: 1,
            name: "Alice".to_string(),
            active: true
        }
    );
    assert_eq!(
        records[1],
        TestRecord {
            id: 2,
            name: "Bob".to_string(),
            active: false
        }
    );
    assert_eq!(
        records[2],
        TestRecord {
            id: 3,
            name: "Charlie".to_string(),
            active: true
        }
    );
}

#[tokio::test]
async fn test_empty_lines_handling() {
    let data = r#"{"value": 1}

{"value": 2}


{"value": 3}

"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<SimpleRecord>();

    let results: Vec<_> = deserializer.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));

    let records: Vec<SimpleRecord> = results.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(records[0], SimpleRecord { value: 1 });
    assert_eq!(records[1], SimpleRecord { value: 2 });
    assert_eq!(records[2], SimpleRecord { value: 3 });
}

#[tokio::test]
async fn test_malformed_json_error() {
    let data = r#"{"value": 1}
{"value": invalid_json}
{"value": 3}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<SimpleRecord>();

    let results: Vec<_> = deserializer.collect().await;

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_err()); // This should be an error due to malformed JSON
    assert!(results[2].is_ok());

    // Check the error message
    let error_msg = results[1].as_ref().unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse JSON line"));
}

#[tokio::test]
async fn test_empty_file() {
    let data = "";

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);

    let results: Vec<_> = iterator.collect().await;

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_single_line() {
    let data = r#"{"value": 42}"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<SimpleRecord>();

    let results: Vec<_> = deserializer.collect().await;

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    let record = results[0].as_ref().unwrap();
    assert_eq!(*record, SimpleRecord { value: 42 });
}

#[tokio::test]
async fn test_type_mismatch_error() {
    let data = r#"{"value": "not_a_number"}
{"value": 42}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let deserializer = iterator.deserialize::<SimpleRecord>();

    let results: Vec<_> = deserializer.collect().await;

    assert_eq!(results.len(), 2);
    assert!(results[0].is_err()); // Type mismatch error
    assert!(results[1].is_ok());

    let record = results[1].as_ref().unwrap();
    assert_eq!(*record, SimpleRecord { value: 42 });
}

#[tokio::test]
async fn test_streaming_behavior() {
    let data = r#"{"value": 1}
{"value": 2}
{"value": 3}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let mut deserializer = iterator.deserialize::<SimpleRecord>();

    // Test that we can consume the stream one item at a time
    let first = deserializer.next().await;
    assert!(first.is_some());
    let first_result = first.unwrap();
    assert!(first_result.is_ok());
    assert_eq!(first_result.unwrap(), SimpleRecord { value: 1 });

    let second = deserializer.next().await;
    assert!(second.is_some());
    let second_result = second.unwrap();
    assert!(second_result.is_ok());
    assert_eq!(second_result.unwrap(), SimpleRecord { value: 2 });

    let third = deserializer.next().await;
    assert!(third.is_some());
    let third_result = third.unwrap();
    assert!(third_result.is_ok());
    assert_eq!(third_result.unwrap(), SimpleRecord { value: 3 });

    let fourth = deserializer.next().await;
    assert!(fourth.is_none());
}

#[tokio::test]
async fn test_jsonl_value_deserialization() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"number": 42, "text": "hello"}
"#;

    let reader = Cursor::new(data.as_bytes());
    let mut stream = Jsonl::new(reader).deserialize_values();

    // Test first value
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first["id"], 1);
    assert_eq!(first["name"], "Alice");
    assert_eq!(first["active"], true);

    // Test second value
    let second = stream.next().await.unwrap().unwrap();
    assert_eq!(second["id"], 2);
    assert_eq!(second["name"], "Bob");
    assert_eq!(second["active"], false);

    // Test third value
    let third = stream.next().await.unwrap().unwrap();
    assert_eq!(third["number"], 42);
    assert_eq!(third["text"], "hello");

    // Test end of stream
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_jsonl_value_trait() {
    let data = r#"{"test": "value"}
{"another": 123}
"#;

    let reader = Cursor::new(data.as_bytes());
    let iterator = Jsonl::new(reader);
    let results: Vec<_> = iterator.deserialize_values().collect().await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));

    let values: Vec<Value> = results.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(values[0]["test"], "value");
    assert_eq!(values[1]["another"], 123);
}

#[tokio::test]
async fn test_jsonl_value_error_handling() {
    let data = r#"{"valid": true}
invalid_json_line
{"also_valid": false}
"#;

    let reader = Cursor::new(data.as_bytes());
    let results: Vec<_> = Jsonl::new(reader).deserialize_values().collect().await;

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_err()); // Invalid JSON
    assert!(results[2].is_ok());

    // Check that valid values are correct
    assert_eq!(results[0].as_ref().unwrap()["valid"], true);
    assert_eq!(results[2].as_ref().unwrap()["also_valid"], false);
}

#[tokio::test]
async fn test_count_empty_file() {
    let data = "";

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_count_single_line() {
    let data = r#"{"value": 42}"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_count_multiple_lines() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
{"id": 4, "name": "Diana", "active": false}
{"id": 5, "name": "Eve", "active": true}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_count_with_empty_lines() {
    let data = r#"{"value": 1}

{"value": 2}


{"value": 3}

"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    // Empty lines should be filtered out, so only 3 valid lines
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_count_large_file() {
    // Create a larger test file with 1000 lines
    let mut data = String::new();
    for i in 0..1000 {
        data.push_str(&format!("{{\"id\": {}, \"value\": \"item_{}\"}}\n", i, i));
    }

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    assert_eq!(count, 1000);
}

#[tokio::test]
async fn test_count_with_malformed_lines() {
    // Count should include all lines, even if they contain invalid JSON
    // because count operates at the line level, not JSON parsing level
    let data = r#"{"valid": 1}
invalid_json_line
{"valid": 2}
another_invalid_line
{"valid": 3}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    let count = JsonlReader::count(jsonl).await;
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_count_consistency_with_streaming() {
    let data = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}
"#;

    // Count using the count method
    let reader1 = Cursor::new(data.as_bytes());
    let jsonl1 = Jsonl::new(reader1);
    let count_method_result = JsonlReader::count(jsonl1).await;

    // Count by manually consuming the stream
    let reader2 = Cursor::new(data.as_bytes());
    let jsonl2 = Jsonl::new(reader2);
    let manual_count = jsonl2.collect::<Vec<_>>().await.len();

    assert_eq!(count_method_result, manual_count);
    assert_eq!(count_method_result, 3);
}

#[tokio::test]
async fn test_complex_nested_values() {
    let data = r#"{"nested": {"inner": [1, 2, 3]}, "top": "level"}
{"array": ["a", "b", "c"], "null_val": null}
"#;

    let reader = Cursor::new(data.as_bytes());
    let results: Vec<_> = Jsonl::new(reader).deserialize_values().collect().await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));

    let values: Vec<Value> = results.into_iter().map(|r| r.unwrap()).collect();

    // Test first complex object
    assert_eq!(values[0]["nested"]["inner"][0], 1);
    assert_eq!(values[0]["nested"]["inner"][1], 2);
    assert_eq!(values[0]["nested"]["inner"][2], 3);
    assert_eq!(values[0]["top"], "level");

    // Test second complex object
    assert_eq!(values[1]["array"][0], "a");
    assert_eq!(values[1]["array"][1], "b");
    assert_eq!(values[1]["array"][2], "c");
    assert!(values[1]["null_val"].is_null());
}

#[tokio::test]
async fn test_take_n_lines_deserialize() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
{"id": 4, "name": "Diana", "active": false}
{"id": 5, "name": "Eve", "active": true}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    // Test first_n with deserialize
    let first_three = jsonl.first_n(3).await.unwrap();
    let records: Vec<TestRecord> = first_three
        .deserialize::<TestRecord>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].id, 1);
    assert_eq!(records[0].name, "Alice");
    assert_eq!(records[1].id, 2);
    assert_eq!(records[1].name, "Bob");
    assert_eq!(records[2].id, 3);
    assert_eq!(records[2].name, "Charlie");
}

#[tokio::test]
async fn test_take_n_lines_reverse_deserialize() {
    let data = r#"{"id": 1, "name": "Alice", "active": true}
{"id": 2, "name": "Bob", "active": false}
{"id": 3, "name": "Charlie", "active": true}
{"id": 4, "name": "Diana", "active": false}
{"id": 5, "name": "Eve", "active": true}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    // Test last_n with deserialize
    let last_two = jsonl.last_n(2).await.unwrap();
    let records: Vec<TestRecord> = last_two
        .deserialize::<TestRecord>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(records.len(), 2);
    // last_n returns in reverse order (last line first)
    assert_eq!(records[0].id, 5);
    assert_eq!(records[0].name, "Eve");
    assert_eq!(records[1].id, 4);
    assert_eq!(records[1].name, "Diana");
}

#[tokio::test]
async fn test_take_n_lines_deserialize_values() {
    let data = r#"{"id": 1, "name": "Alice"}
{"id": 2, "name": "Bob"}
{"id": 3, "name": "Charlie"}
"#;

    let reader = Cursor::new(data.as_bytes());
    let jsonl = Jsonl::new(reader);

    // Test first_n with deserialize_values
    let first_two = jsonl.first_n(2).await.unwrap();
    let values: Vec<Value> = first_two
        .deserialize_values()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["id"], 1);
    assert_eq!(values[0]["name"], "Alice");
    assert_eq!(values[1]["id"], 2);
    assert_eq!(values[1]["name"], "Bob");
}
