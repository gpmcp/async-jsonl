use crate::buf_reader::RevBufReader;
use std::io::Result as IoResult;
use tokio::io::{AsyncRead, AsyncSeek};

/// Optimized streaming interface for reading lines in reverse
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Lines<R> {
    reader: RevBufReader<R>,
}

impl<R> Lines<R> {
    pub(crate) fn new(reader: RevBufReader<R>) -> Self {
        Self { reader }
    }
}

impl<R: AsyncRead + AsyncSeek + Unpin> Lines<R> {
    /// Returns the next line in reverse order, following tokio patterns
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use async_rev_buf::RevBufReader;
    /// use std::io::Cursor;
    /// 
    /// async fn example() -> std::io::Result<()> {
    ///     let data = "first line\nsecond line\nthird line";
    ///     let cursor = Cursor::new(data);
    ///     let reader = RevBufReader::new(cursor);
    ///     let mut lines = reader.lines();
    /// 
    ///     // Read lines in reverse order
    ///     assert_eq!(lines.next_line().await?, Some("third line".to_string()));
    ///     assert_eq!(lines.next_line().await?, Some("second line".to_string()));
    ///     assert_eq!(lines.next_line().await?, Some("first line".to_string()));
    ///     assert_eq!(lines.next_line().await?, None);
    ///     Ok(())
    /// }
    /// ```
    pub async fn next_line(&mut self) -> IoResult<Option<String>> {
        self.reader.next_line().await
    }

    /// Returns a reference to the underlying `RevBufReader`
    pub fn get_ref(&self) -> &RevBufReader<R> {
        &self.reader
    }

    /// Returns a mutable reference to the underlying `RevBufReader`
    pub fn get_mut(&mut self) -> &mut RevBufReader<R> {
        &mut self.reader
    }

    /// Consumes this `Lines`, returning the underlying reader
    pub fn into_inner(self) -> RevBufReader<R> {
        self.reader
    }
}