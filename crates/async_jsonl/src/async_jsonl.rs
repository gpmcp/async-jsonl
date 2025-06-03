use futures::Stream;
use serde::Deserialize;
use serde_json::Value;
use tokio::io::{BufReader, Lines};

/// Iterator to read JSONL file as raw JSON strings
pub struct Jsonl<R> {
    pub(crate) lines: Lines<BufReader<R>>,
}

#[async_trait::async_trait]
pub trait JsonlReader: JsonlDeserialize + JsonlValueDeserialize + Stream + Send + Sync {
    type NLines: Stream<Item = anyhow::Result<String>>;
    type NLinesRev: Stream<Item = anyhow::Result<String>>;
    async fn first_n(self, n: usize) -> anyhow::Result<Self::NLines>;
    async fn last_n(self, n: usize) -> anyhow::Result<Self::NLinesRev>;
}

/// Extension trait to add deserialization capabilities to JsonlIterator
pub trait JsonlDeserialize {
    /// Deserialize JSON lines into the specified type
    fn deserialize<T>(self) -> impl Stream<Item = anyhow::Result<T>>
    where
        T: for<'a> Deserialize<'a>;
}

/// Extension trait specifically for deserializing JSONL to serde_json::Value
pub trait JsonlValueDeserialize {
    /// Deserialize JSON lines into serde_json::Value objects
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>>;
}
