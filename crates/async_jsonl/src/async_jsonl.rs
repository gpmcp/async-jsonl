use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader, Lines};

/// Iterator to read JSONL file as raw JSON strings
pub struct Jsonl<R> {
    pub(crate) lines: Lines<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> Jsonl<R> {
    pub fn new(file: R) -> Self {
        let reader = BufReader::new(file);
        Self {
            lines: reader.lines(),
        }
    }
    /// Count lines from any AsyncRead source
    pub async fn count_lines(mut self) -> anyhow::Result<usize> {
        let mut count = 0;
        while let Some(line) = self.lines.next_line().await? {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                count += 1;
            }
        }
        Ok(count)
    }
}

impl Jsonl<File> {
    /// Create a new Jsonl reader from a file path
    pub async fn from_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::open(path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open file: {}", e))?;
        Ok(Self::new(file))
    }
}

impl<R: AsyncRead + Unpin> Stream for Jsonl<R> {
    type Item = anyhow::Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.lines).poll_next_line(cx) {
            Poll::Ready(Ok(Some(line))) => {
                let line = line.trim();
                if line.is_empty() {
                    // Skip empty lines and recursively poll for next
                    self.poll_next(cx)
                } else {
                    Poll::Ready(Some(Ok(line.to_string())))
                }
            }
            Poll::Ready(Ok(None)) => Poll::Ready(None), // EOF
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(anyhow::anyhow!("IO error: {}", e)))),
            Poll::Pending => Poll::Pending,
        }
    }
}
