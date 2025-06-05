use tokio::io::AsyncBufReadExt;
use async_rev_buf::RevBufReader;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use rev_buf_reader::RevBufReader as SyncRevBufReader;
use std::io::{BufRead, Cursor};
use tokio::io::BufReader;
use tokio::runtime::Runtime;

fn create_test_data(num_lines: usize) -> String {
    (0..num_lines)
        .map(|i| format!("This is line number {} with some content to make it realistic", i))
        .collect::<Vec<_>>()
        .join("\n")
}

fn bench_async_vs_sync_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_vs_sync_reverse_reading");
    let rt = Runtime::new().unwrap();

    for &num_lines in &[100, 1000, 5000] {
        let test_data = create_test_data(num_lines);
        group.throughput(Throughput::Elements(num_lines as u64));
        
        // Test fwd read using tokio's BufReader
        group.bench_with_input(
            BenchmarkId::new("async_tokio_stream", num_lines),
            &test_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let reader = BufReader::new(Cursor::new(data.as_bytes()));
                        let mut lines = reader.lines();
                        let mut count = 0;
                        
                        while let Some(line) = lines.next_line().await.unwrap() {
                            black_box(line);
                            count += 1;
                        }
                        count
                    })
                })
            },
        );
        
        // Test our async implementation (direct method)
        group.bench_with_input(
            BenchmarkId::new("rev_bufreader", num_lines),
            &test_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut reader = RevBufReader::new(Cursor::new(data.as_bytes())).lines();
                        let mut count = 0;
                        
                        while let Some(line) = reader.next_line().await.unwrap() {
                            black_box(line);
                            count += 1;
                        }
                        count
                    })
                })
            },
        );
        
        // Test crates.io sync implementation
        group.bench_with_input(
            BenchmarkId::new("sync_crates_io", num_lines),
            &test_data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(data.as_bytes());
                    let reader = SyncRevBufReader::new(cursor);
                    let mut count = 0;
                    
                    for _line in reader.lines() {
                        black_box(_line.unwrap());
                        count += 1;
                    }
                    count
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_async_vs_sync_comparison);
criterion_main!(benches);