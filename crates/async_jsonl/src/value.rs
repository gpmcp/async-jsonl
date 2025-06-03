use crate::Jsonl;
use futures::{Stream, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncRead;

/// Extension trait to add deserialization capabilities to JsonlIterator
pub trait JsonlDeserialize<R> {
    /// Deserialize JSON lines into the specified type
    fn deserialize<T>(self) -> impl Stream<Item = anyhow::Result<T>>
    where
        T: for<'a> Deserialize<'a>;
}

impl<R: AsyncRead + Unpin> JsonlDeserialize<R> for Jsonl<R> {
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

/// Extension trait specifically for deserializing JSONL to serde_json::Value
pub trait JsonlValueDeserialize<R> {
    /// Deserialize JSON lines into serde_json::Value objects
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>>;
}

impl<R: AsyncRead + Unpin> JsonlValueDeserialize<R> for Jsonl<R> {
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>> {
        self.deserialize::<Value>()
    }
}

/// Convenience function to create a JsonlIterator that specifically works with serde_json::Value
pub fn jsonl_values<R>(reader: R) -> impl Stream<Item = anyhow::Result<Value>>
where
    R: AsyncRead + Unpin,
{
    Jsonl::new(reader).deserialize_values()
}
