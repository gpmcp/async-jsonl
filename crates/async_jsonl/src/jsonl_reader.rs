use crate::take_n::{TakeNLines, TakeNLinesReverse};
use crate::{Jsonl, JsonlReader};
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncSeek, BufReader};

#[async_trait::async_trait]
impl<R: AsyncRead + AsyncSeek + Unpin + Sync + Send> JsonlReader for Jsonl<R> {
    type NLines = TakeNLines<R>;
    type NLinesRev = TakeNLinesReverse;

    async fn first_n(self, n: usize) -> anyhow::Result<Self::NLines> {
        Ok(self.get_n(n))
    }

    async fn last_n(self, n: usize) -> anyhow::Result<Self::NLinesRev> {
        self.get_rev_n(n).await
    }
}

impl<R: AsyncRead + Unpin> Jsonl<R> {
    pub fn new(file: R) -> Self {
        let reader = BufReader::new(file);
        Self {
            lines: reader.lines(),
        }
    }

    /// Get the first n lines from the beginning of the file
    pub(crate) fn get_n(self, n: usize) -> TakeNLines<R> {
        let reader = self.lines.into_inner().into_inner();
        TakeNLines::new(reader, n)
    }
}

impl<R: AsyncRead + AsyncSeek + Unpin> Jsonl<R> {
    /// Get the last n lines from the end of the file (like tail)
    pub(crate) async fn get_rev_n(self, n: usize) -> anyhow::Result<TakeNLinesReverse> {
        let reader = self.lines.into_inner().into_inner();
        TakeNLinesReverse::new(reader, n).await
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
