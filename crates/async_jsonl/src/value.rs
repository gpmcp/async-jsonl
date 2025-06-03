use crate::{JsonlDeserialize, Jsonl};
use futures::Stream;
use serde_json::Value;
use tokio::io::AsyncRead;

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
pub fn jsonl_values<R: AsyncRead>(reader: R) -> impl Stream<Item = anyhow::Result<Value>>
where
    R: Unpin,
{
    Jsonl::new(reader).deserialize_values()
}

