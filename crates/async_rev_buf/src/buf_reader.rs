use crate::DEFAULT_BUF_SIZE;
use pin_project_lite::pin_project;
use std::io::{Result as IoResult, SeekFrom};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, ReadBuf};

pin_project! {
    /// A high-performance buffered reader that reads lines in reverse order.
    #[derive(Debug)]
    pub struct RevBufReader<R> {
        #[pin]
        inner: R,
        buf: Box<[u8]>,
        pos: usize,        // Current position in buffer (reading from end to start)
        cap: usize,        // Amount of valid data in buffer
        file_pos: u64,     // Current position in file
        file_size: u64,    // Total file size (cached)
        initialized: bool, // Whether we've initialized file size
    }
}

impl<R: AsyncRead> RevBufReader<R> {
    /// Creates a new reverse buffered reader with default capacity.
    pub fn new(inner: R) -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a new reverse buffered reader with the specified capacity.
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self {
            inner,
            buf: vec![0; capacity].into_boxed_slice(),
            pos: 0,
            cap: 0,
            file_pos: 0,
            file_size: 0,
            initialized: false,
        }
    }

    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Gets a pinned mutable reference to the underlying reader.
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().inner
    }

    /// Consumes this reader, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }
}

impl<R: AsyncRead + AsyncSeek + Unpin> RevBufReader<R> {
    /// Initialize file size if not already done - optimized to do this once
    async fn ensure_initialized(&mut self) -> IoResult<()> {
        if !self.initialized {
            self.file_size = self.inner.seek(SeekFrom::End(0)).await?;
            self.file_pos = self.file_size;
            self.initialized = true;
        }
        Ok(())
    }

    /// Efficiently seek backward in the file
    async fn seek_back(&mut self, length: usize) -> IoResult<usize> {
        if self.file_pos == 0 {
            return Ok(0);
        }
        
        // Calculate how much we can actually seek back
        let seek_amount = std::cmp::min(length as u64, self.file_pos) as usize;
        let new_pos = self.file_pos - seek_amount as u64;
        
        // Seek to the new position
        self.inner.seek(SeekFrom::Start(new_pos)).await?;
        self.file_pos = new_pos;
        self.cap = 0;
        self.pos = 0;
        
        Ok(seek_amount)
    }

    /// Fill the buffer with data from the current position
    async fn fill_buffer(&mut self) -> IoResult<&[u8]> {
        if self.pos == 0 {
            let length = self.seek_back(self.buf.len()).await?;
            if length == 0 {
                return Ok(&[]);
            }
            
            // Read the data from current position
            let mut total_read = 0;
            while total_read < length {
                match self.inner.read(&mut self.buf[total_read..length]).await? {
                    0 => break,
                    n => total_read += n,
                }
            }
            
            self.cap = total_read;
            self.pos = total_read; // Start from the end of the buffer
        }
        Ok(&self.buf[0..self.pos])
    }

    /// Consume bytes from the buffer (moving backward)
    fn consume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_sub(amt);
    }

    /// Core optimized line reading implementation
    async fn read_line_internal(&mut self, buf: &mut String) -> IoResult<usize> {
        self.ensure_initialized().await?;
        
        if self.file_size == 0 {
            return Ok(0);
        }

        let mut line_buffer = Vec::new();

        loop {
            // Get buffer data efficiently
            let (buffer_slice, current_pos) = {
                let buffer_data = self.fill_buffer().await?;
                if buffer_data.is_empty() {
                    break;
                }
                (buffer_data.to_vec(), self.pos)
            };

            // Search for newline from the end
            if let Some(newline_pos) = buffer_slice.iter().rposition(|&b| b == b'\n' || b == b'\r') {
                // Found a newline - extract the line after it
                let line_start = newline_pos + 1;
                let line_data = &buffer_slice[line_start..current_pos];
                
                // Build the line (prepend since we're reading backward)
                let mut new_line = line_data.to_vec();
                new_line.extend_from_slice(&line_buffer);
                line_buffer = new_line;

                // Consume up to and including the newline
                self.consume(current_pos - newline_pos);

                // Convert to string
                let line_str = String::from_utf8_lossy(&line_buffer);
                let trimmed = line_str.trim_end_matches('\r');
                if !trimmed.is_empty() {
                    buf.push_str(trimmed);
                    return Ok(trimmed.len());
                }
                // Empty line, continue to next
                line_buffer.clear();
            } else {
                // No newline found - consume entire buffer
                let mut new_line = buffer_slice;
                new_line.extend_from_slice(&line_buffer);
                line_buffer = new_line;
                
                self.consume(current_pos);
                
                if self.file_pos == 0 && self.pos == 0 {
                    // Reached start of file
                    if !line_buffer.is_empty() {
                        let line_str = String::from_utf8_lossy(&line_buffer);
                        let trimmed = line_str.trim_end_matches('\r');
                        buf.push_str(trimmed);
                        return Ok(trimmed.len());
                    }
                    break;
                }
            }
        }

        Ok(0)
    }

    /// Read the next line in reverse order
    pub async fn next_line(&mut self) -> IoResult<Option<String>> {
        let mut line = String::new();
        match self.read_line_internal(&mut line).await? {
            0 => Ok(None),
            _ => Ok(Some(line)),
        }
    }

    /// Returns a stream of lines read in reverse order
    pub fn lines(self) -> crate::Lines<R>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        crate::Lines::new(self)
    }
}

// AsyncRead implementation for completeness
impl<R: AsyncRead + Unpin> AsyncRead for RevBufReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Poll::Ready(Ok(()))
    }
}