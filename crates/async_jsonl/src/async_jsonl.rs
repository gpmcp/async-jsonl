use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncSeek, BufReader, Lines};

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

/// Stream that yields n lines from the beginning of a JSONL file
pub struct TakeNLines<R> {
    lines: Lines<BufReader<R>>,
    remaining: usize,
}

impl<R: AsyncRead + Unpin> TakeNLines<R> {
    fn new(reader: R, n: usize) -> Self {
        let buf_reader = BufReader::new(reader);
        Self {
            lines: buf_reader.lines(),
            remaining: n,
        }
    }
}

impl<R: AsyncRead + Unpin> Stream for TakeNLines<R> {
    type Item = anyhow::Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.lines).poll_next_line(cx) {
            Poll::Ready(Ok(Some(line))) => {
                let line = line.trim();
                if !line.is_empty() {
                    self.remaining -= 1;
                    Poll::Ready(Some(Ok(line.to_string())))
                } else {
                    // Skip empty lines and try again
                    self.poll_next(cx)
                }
            }
            Poll::Ready(Ok(None)) => Poll::Ready(None), // EOF
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(anyhow::anyhow!("IO error: {}", e)))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream that yields n lines from the end of a JSONL file
pub struct TakeNLinesReverse {
    lines: std::vec::IntoIter<String>,
}

impl TakeNLinesReverse {
    async fn new<R: AsyncRead + AsyncSeek + Unpin>(
        mut reader: R,
        n: usize,
    ) -> std::io::Result<Self> {
        let mut lines_found = Vec::new();
        let mut buffer = Vec::new();
        let chunk_size = 8192;

        let file_size =
            tokio::io::AsyncSeekExt::seek(&mut reader, std::io::SeekFrom::End(0)).await?;

        if file_size == 0 || n == 0 {
            return Ok(Self {
                lines: Vec::new().into_iter(),
            });
        }

        let mut current_pos = file_size;

        // Read file backwards until we find n lines
        while current_pos > 0 && lines_found.len() < n {
            let read_size = std::cmp::min(chunk_size as u64, current_pos) as usize;
            let new_pos = current_pos - read_size as u64;

            tokio::io::AsyncSeekExt::seek(&mut reader, std::io::SeekFrom::Start(new_pos)).await?;

            let mut chunk = vec![0u8; read_size];
            tokio::io::AsyncReadExt::read_exact(&mut reader, &mut chunk).await?;

            chunk.extend_from_slice(&buffer);
            buffer = chunk;
            current_pos = new_pos;

            let buffer_str = String::from_utf8_lossy(&buffer).into_owned();
            let lines: Vec<&str> = buffer_str.lines().collect();

            let start_idx = if current_pos > 0 && !buffer.is_empty() && buffer[0] != b'\n' {
                if lines.len() > 1 {
                    let incomplete_line = lines[0].to_string();
                    buffer = incomplete_line.into_bytes();
                    1
                } else {
                    continue;
                }
            } else {
                buffer.clear();
                0
            };

            for line in lines[start_idx..].iter().rev() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    lines_found.insert(0, trimmed.to_string());
                    if lines_found.len() >= n {
                        break;
                    }
                }
            }
        }

        // Keep only the last n lines and reverse to get correct order (last line first)
        if lines_found.len() > n {
            let excess = lines_found.len() - n;
            lines_found.drain(0..excess);
        }
        lines_found.reverse();

        Ok(Self {
            lines: lines_found.into_iter(),
        })
    }
}

impl Stream for TakeNLinesReverse {
    type Item = anyhow::Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.lines.next() {
            Some(line) => Poll::Ready(Some(Ok(line))),
            None => Poll::Ready(None),
        }
    }
}

// Add get_n and get_rev_n methods
impl<R: AsyncRead + Unpin> Jsonl<R> {
    /// Get the first n lines from the beginning of the file
    pub fn get_n(self, n: usize) -> TakeNLines<R> {
        let reader = self.lines.into_inner().into_inner();
        TakeNLines::new(reader, n)
    }
}

impl<R: AsyncRead + AsyncSeek + Unpin> Jsonl<R> {
    /// Get the last n lines from the end of the file (like tail)
    pub async fn get_rev_n(self, n: usize) -> std::io::Result<TakeNLinesReverse> {
        let reader = self.lines.into_inner().into_inner();
        TakeNLinesReverse::new(reader, n).await
    }
}

// Special implementations for File
impl Jsonl<File> {
    /// Get the first n lines from the beginning of the file
    pub fn get_n_file(self, n: usize) -> TakeNLines<File> {
        let reader = self.lines.into_inner().into_inner();
        TakeNLines::new(reader, n)
    }
}
