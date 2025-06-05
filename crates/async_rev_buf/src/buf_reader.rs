use pin_project_lite::pin_project;
use std::io::{SeekFrom, Result as IoResult};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, ReadBuf};
use crate::{DEFAULT_BUF_SIZE, Lines};

pin_project! {
    /// A buffered reader that reads lines in reverse order from the end of the input.
    #[derive(Debug)]
    pub struct RevBufReader<R> {
        #[pin]
        inner: R,
        buf: Box<[u8]>,
        pos: usize,        // Current position in buffer
        cap: usize,        // Amount of valid data in buffer
        file_pos: Option<u64>,  // Current position in file (None means uninitialized)
        file_size: Option<u64>, // Total file size (cached)
        at_start: bool,    // Whether we've reached the start of the file
    }
}



impl<R: AsyncRead> RevBufReader<R> {
    /// Creates a new `BufReader` with a default buffer capacity. The default is currently 8 KB,
    /// but may change in the future.
    pub fn new(inner: R) -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a new `BufReader` with the specified buffer capacity.
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self {
            inner,
            buf: vec![0; capacity].into_boxed_slice(),
            pos: 0,
            cap: 0,
            file_pos: None,
            file_size: None,
            at_start: false,
        }
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Gets a pinned mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().inner
    }

    /// Consumes this `BufReader`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Returns a reference to the internally buffered data.
    ///
    /// Unlike `fill_buf`, this will not attempt to fill the buffer if it is empty.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }


}

impl<R: AsyncRead + AsyncSeek + Unpin> RevBufReader<R> {
    /// Initialize file position and size if not already done
    async fn initialize(&mut self) -> IoResult<()> {
        if self.file_size.is_none() {
            self.file_size = Some(self.inner.seek(SeekFrom::End(0)).await?);
            self.file_pos = self.file_size;
        }
        Ok(())
    }
}

// AsyncRead implementation (for completeness, though not used for reverse reading)
impl<R: AsyncRead + Unpin> AsyncRead for RevBufReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        // For reverse reading, we don't implement forward reading
        // This is primarily to satisfy trait bounds if needed
        Poll::Ready(Ok(()))
    }
}

// Note: We don't implement AsyncBufRead because it doesn't make sense for reverse reading
// Instead, we provide our own interface through Lines

impl<R: AsyncRead + AsyncSeek + Unpin> RevBufReader<R> {
    /// Get the next line from the file reading in reverse
    pub async fn poll_next_line_reverse(&mut self) -> IoResult<Option<String>> {
        // Initialize once
        self.initialize().await?;
        
        let file_size = self.file_size.unwrap();
        if file_size == 0 {
            return Ok(None);
        }
        
        // If this is the first call, position at end of file
        if self.file_pos.is_none() {
            self.file_pos = Some(file_size);
        }
        
        let mut accumulated_data = Vec::new();
        let mut current_end = self.file_pos.unwrap();
        
        while current_end > 0 {
            // Calculate chunk size
            let chunk_size = std::cmp::min(self.buf.len() as u64, current_end) as usize;
            let chunk_start = current_end - chunk_size as u64;
            
            // Read chunk
            self.inner.seek(SeekFrom::Start(chunk_start)).await?;
            let mut chunk = vec![0u8; chunk_size];
            let mut total_read = 0;
            while total_read < chunk_size {
                match self.inner.read(&mut chunk[total_read..chunk_size]).await? {
                    0 => break,
                    n => total_read += n,
                }
            }
            chunk.truncate(total_read);
            
            // Prepend to accumulated data
            let mut new_data = chunk;
            new_data.extend_from_slice(&accumulated_data);
            accumulated_data = new_data;
            
            // Look for lines in accumulated data
            let text = String::from_utf8_lossy(&accumulated_data);
            let lines: Vec<&str> = text.lines().collect();
            
            if lines.len() > 1 || (lines.len() == 1 && chunk_start == 0) {
                // We have at least one complete line
                let last_line = lines[lines.len() - 1].trim();
                
                if !last_line.is_empty() {
                    // Calculate where this line ends in the file
                    if lines.len() > 1 {
                        // There are more lines before this one
                        let before_last = &lines[0..lines.len() - 1];
                        let before_text = before_last.join("\n") + "\n";
                        self.file_pos = Some(chunk_start + before_text.as_bytes().len() as u64);
                    } else {
                        // This is the only/first line
                        self.file_pos = Some(chunk_start);
                    }
                    
                    return Ok(Some(last_line.to_string()));
                }
                
                // Empty line, continue to previous
                if lines.len() > 1 {
                    let before_last = &lines[0..lines.len() - 1];
                    let before_text = before_last.join("\n") + "\n";
                    self.file_pos = Some(chunk_start + before_text.as_bytes().len() as u64);
                    accumulated_data.clear();
                    current_end = self.file_pos.unwrap();
                    continue;
                }
            }
            
            // Need more data
            current_end = chunk_start;
            
            if chunk_start == 0 {
                // We've reached the beginning
                if !accumulated_data.is_empty() {
                    let text = String::from_utf8_lossy(&accumulated_data);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        self.file_pos = Some(0);
                        return Ok(Some(trimmed.to_string()));
                    }
                }
                return Ok(None);
            }
        }
        
        Ok(None)
    }
    
    /// Returns a stream of lines read in reverse order
    pub fn lines(self) -> Lines<Self>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        Lines::new(self)
    }
}
