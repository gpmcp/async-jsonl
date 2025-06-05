use crate::RevBufReader;
use std::io::Result as IoResult;
use tokio::io::{AsyncRead, AsyncSeek};

/// Reads lines from a [`RevBufReader`] in reverse order.
///
/// This type is usually created using the [`lines`] method.
///
/// [`lines`]: RevBufReader::lines
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Lines<R> {
    reader: R,
}

impl<R> Lines<R> {
    pub(crate) fn new(reader: R) -> Self {
        Lines { reader }
    }
}

impl<R> Lines<RevBufReader<R>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    /// Returns the next line in the stream (reading backwards).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_jsonl_rev_buf::RevBufReader;
    /// # use std::io::Cursor;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let data = b"Line 1\nLine 2\nLine 3\n";
    /// let cursor = Cursor::new(data);
    /// let reader = RevBufReader::new(cursor);
    /// let mut lines = reader.lines();
    ///
    /// while let Some(line) = lines.next_line().await? {
    ///     println!("{}", line); // Prints Line 3, Line 2, Line 1
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next_line(&mut self) -> IoResult<Option<String>> {
        self.reader.poll_next_line_reverse().await
    }

    /// Obtains a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut RevBufReader<R> {
        &mut self.reader
    }

    /// Obtains a reference to the underlying reader.
    pub fn get_ref(&self) -> &RevBufReader<R> {
        &self.reader
    }

    /// Unwraps this `Lines<RevBufReader<R>>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    /// Therefore, a following read from the underlying reader may lead to data loss.
    pub fn into_inner(self) -> RevBufReader<R> {
        self.reader
    }
}
