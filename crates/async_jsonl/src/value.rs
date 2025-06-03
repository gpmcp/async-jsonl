use crate::take_n::{TakeNLines, TakeNLinesReverse};
use crate::{Jsonl, JsonlDeserialize, JsonlValueDeserialize};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncRead;

impl<R: AsyncRead + Unpin> JsonlDeserialize for Jsonl<R> {
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

impl<R: AsyncRead + Unpin> JsonlValueDeserialize for Jsonl<R> {
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>> {
        self.deserialize::<Value>()
    }
}

// Implementations for TakeNLines
impl<R: AsyncRead + Unpin> JsonlDeserialize for TakeNLines<R> {
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

impl<R: AsyncRead + Unpin> JsonlValueDeserialize for TakeNLines<R> {
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>> {
        self.deserialize::<Value>()
    }
}

// Implementations for TakeNLinesReverse
impl JsonlDeserialize for TakeNLinesReverse {
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

impl JsonlValueDeserialize for TakeNLinesReverse {
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>> {
        self.deserialize::<Value>()
    }
}
