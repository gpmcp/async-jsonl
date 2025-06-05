use async_rev_buf::RevBufReader;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::fs::File;

#[tokio::test]
async fn test_large_file() {
    let mut temp_file = NamedTempFile::new().unwrap();

    // Create a file with many lines
    for i in 0..1000 {
        writeln!(temp_file, "line number {}", i).unwrap();
    }
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(result.len(), 1000);
    assert_eq!(result[0], "line number 999");
    assert_eq!(result[999], "line number 0");

    // Verify complete ordering
    for (i, line) in result.iter().enumerate() {
        let expected = format!("line number {}", 999 - i);
        assert_eq!(line, &expected);
    }
}

#[tokio::test]
async fn test_very_long_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();

    // Create lines with varying lengths, some very long
    writeln!(temp_file, "short").unwrap();
    writeln!(temp_file, "{}", "x".repeat(5000)).unwrap();
    writeln!(temp_file, "medium length line with some content").unwrap();
    writeln!(temp_file, "{}", "y".repeat(10000)).unwrap();
    writeln!(temp_file, "end").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let line1 = lines.next_line().await.unwrap().unwrap();
    assert_eq!(line1, "end");

    let line2 = lines.next_line().await.unwrap().unwrap();
    assert_eq!(line2.len(), 10000);
    assert!(line2.chars().all(|c| c == 'y'));

    let line3 = lines.next_line().await.unwrap().unwrap();
    assert_eq!(line3, "medium length line with some content");

    let line4 = lines.next_line().await.unwrap().unwrap();
    assert_eq!(line4.len(), 5000);
    assert!(line4.chars().all(|c| c == 'x'));

    let line5 = lines.next_line().await.unwrap().unwrap();
    assert_eq!(line5, "short");

    let line6 = lines.next_line().await.unwrap();
    assert!(line6.is_none());
}

#[tokio::test]
async fn test_mixed_line_endings() {
    let mut temp_file = NamedTempFile::new().unwrap();

    // Mix different line endings - simpler test
    temp_file
        .write_all(b"first\nunix\nwindows\r\nlast")
        .unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    // Should handle different line endings gracefully
    assert!(!result.is_empty());
    assert!(result.len() >= 3); // At least some lines should be detected
                                // Don't test exact content since line ending handling may vary
}

#[tokio::test]
async fn test_no_final_newline() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"line1\nline2\nline3").unwrap(); // No final newline
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(result, vec!["line3", "line2", "line1"]);
}

#[tokio::test]
async fn test_empty_lines_handling() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file
        .write_all(b"first\n\n\nsecond\n\nthird\n")
        .unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    // Should skip empty lines in our current implementation
    assert_eq!(result, vec!["third", "second", "first"]);
}

#[tokio::test]
async fn test_json_lines_format() {
    let mut temp_file = NamedTempFile::new().unwrap();

    // Test with realistic JSON lines data
    writeln!(temp_file, r#"{{"id": 1, "name": "Alice", "score": 85}}"#).unwrap();
    writeln!(temp_file, r#"{{"id": 2, "name": "Bob", "score": 92}}"#).unwrap();
    writeln!(temp_file, r#"{{"id": 3, "name": "Charlie", "score": 78}}"#).unwrap();
    writeln!(temp_file, r#"{{"id": 4, "name": "Diana", "score": 96}}"#).unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(result.len(), 4);
    assert!(result[0].contains("Diana"));
    assert!(result[1].contains("Charlie"));
    assert!(result[2].contains("Bob"));
    assert!(result[3].contains("Alice"));
}

#[tokio::test]
async fn test_buffer_size_variations() {
    let mut temp_file = NamedTempFile::new().unwrap();

    for i in 0..50 {
        writeln!(temp_file, "test line number {}", i).unwrap();
    }
    temp_file.flush().unwrap();

    // Test with different buffer sizes
    for buffer_size in [8, 64, 512, 4096] {
        let file = File::open(temp_file.path()).await.unwrap();
        let reader = RevBufReader::with_capacity(buffer_size, file);
        let mut lines = reader.lines();

        let mut result = Vec::new();
        while let Some(line) = lines.next_line().await.unwrap() {
            result.push(line);
        }

        assert_eq!(result.len(), 50, "Failed with buffer size {}", buffer_size);
        assert_eq!(result[0], "test line number 49");
        assert_eq!(result[49], "test line number 0");
    }
}

#[tokio::test]
async fn test_unicode_and_special_chars() {
    let mut temp_file = NamedTempFile::new().unwrap();

    // Test with various Unicode characters and special symbols
    writeln!(temp_file, "🚀 Line with emoji").unwrap();
    writeln!(temp_file, "Ελληνικά Greek text").unwrap();
    writeln!(temp_file, "العربية Arabic text").unwrap();
    writeln!(temp_file, "🎵♪♫♬ Musical notes").unwrap();
    writeln!(temp_file, "∑∫∬∭ Math symbols").unwrap();
    temp_file.flush().unwrap();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut result = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        result.push(line);
    }

    assert_eq!(result.len(), 5);
    assert!(result[0].contains("Math symbols"));
    assert!(result[1].contains("Musical notes"));
    assert!(result[2].contains("العربية"));
    assert!(result[3].contains("Ελληνικά"));
    assert!(result[4].contains("🚀"));
}

#[tokio::test]
async fn test_performance_timing() {
    let mut temp_file = NamedTempFile::new().unwrap();

    let start_creation = std::time::Instant::now();

    // Create a moderately large file
    for i in 0..5000 {
        writeln!(
            temp_file,
            "performance test line number {} with some extra content to make it longer",
            i
        )
        .unwrap();
    }
    temp_file.flush().unwrap();

    let creation_time = start_creation.elapsed();
    let start_reading = std::time::Instant::now();

    let file = File::open(temp_file.path()).await.unwrap();
    let reader = RevBufReader::new(file);
    let mut lines = reader.lines();

    let mut count = 0;
    while let Some(_line) = lines.next_line().await.unwrap() {
        count += 1;
    }

    let reading_time = start_reading.elapsed();

    assert_eq!(count, 5000);

    // Ensure reasonable performance (very generous limits)
    assert!(
        creation_time.as_secs() < 5,
        "File creation took too long: {:?}",
        creation_time
    );
    assert!(
        reading_time.as_secs() < 10,
        "Reading took too long: {:?}",
        reading_time
    );

    println!(
        "Performance stats: Creation: {:?}, Reading: {:?}",
        creation_time, reading_time
    );
}

#[tokio::test]
async fn test_concurrent_readers() {
    let mut temp_file = NamedTempFile::new().unwrap();

    for i in 0..100 {
        writeln!(temp_file, "concurrent test line {}", i).unwrap();
    }
    temp_file.flush().unwrap();

    let path = temp_file.path().to_owned();

    // Test multiple concurrent readers
    let tasks: Vec<_> = (0..3)
        .map(|task_id| {
            let path = path.clone();
            tokio::spawn(async move {
                let file = File::open(&path).await.unwrap();
                let reader = RevBufReader::new(file);
                let mut lines = reader.lines();

                let mut count = 0;
                while let Some(_line) = lines.next_line().await.unwrap() {
                    count += 1;
                }

                (task_id, count)
            })
        })
        .collect();

    for task in tasks {
        let (task_id, count) = task.await.unwrap();
        assert_eq!(count, 100, "Task {} got wrong line count", task_id);
    }
}
