use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader, Lines};

/// Iterator to read JSONL file as raw JSON strings
pub struct JsonlIterator<R> {
    pub(crate) lines: Lines<BufReader<R>>,
}

impl<R: AsyncRead> JsonlIterator<R> {
    pub fn new(file: R) -> Self {
        let reader = BufReader::new(file);
        Self {
            lines: reader.lines(),
        }
    }
}

impl<R: AsyncRead + Unpin> Stream for JsonlIterator<R> {
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

/// Extension trait to add deserialization capabilities to JsonlIterator
pub trait JsonlDeserialize<R> {
    /// Deserialize JSON lines into the specified type
    fn deserialize<T>(self) -> impl Stream<Item = anyhow::Result<T>>
    where
        T: for<'a> Deserialize<'a>;
}

impl<R: AsyncRead + Unpin> JsonlDeserialize<R> for JsonlIterator<R> {
    fn deserialize<T>(self) -> impl Stream<Item = anyhow::Result<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        self.map(|result| {
            result.and_then(|line| {
                serde_json::from_str::<T>(&line)
                    .map_err(|e| anyhow::anyhow!("Failed to parse JSON line: {}", e))
            })
        })
    }
}
