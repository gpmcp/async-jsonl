use futures::Stream;
use serde::Deserialize;
use serde_json::Value;
use tokio::io::{BufReader, Lines};

/// Iterator to read JSONL file as raw JSON strings
pub struct Jsonl<R> {
    pub(crate) lines: Lines<BufReader<R>>,
}

/// Main trait for reading JSONL (JSON Lines) files with async capabilities.
///
/// This trait provides methods to read and process JSONL files asynchronously.
/// It combines streaming capabilities with deserialization and line selection methods.
/// The trait is implemented by `Jsonl<R>` where `R` implements `AsyncRead + AsyncSeek`.
///
/// # Examples
///
/// ## Reading from a file and getting first n lines
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlReader, JsonlDeserialize};
/// use futures::StreamExt;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let reader = Jsonl::from_path("people.jsonl").await?;
///     
///     // Get first 5 lines and deserialize directly
///     let first_five = reader.first_n(5).await?;
///     let mut stream = first_five.deserialize::<Person>();
///     
///     while let Some(result) = stream.next().await {
///         match result {
///             Ok(person) => println!("Found person: {:?}", person),
///             Err(e) => eprintln!("Error parsing line: {}", e),
///         }
///     }
///     
///     Ok(())
/// }
/// ```
///
/// ## Reading last n lines (tail-like functionality)
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlReader};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let reader = Jsonl::from_path("log.jsonl").await?;
///     
///     // Get last 10 lines (like tail)
///     let last_ten = reader.last_n(10).await?;
///     
///     let lines: Vec<String> = last_ten
///         .collect::<Vec<_>>()
///         .await
///         .into_iter()
///         .collect::<Result<Vec<_>, _>>()?;
///     
///     for line in lines {
///         println!("{}", line);
///     }
///     
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait JsonlReader: JsonlDeserialize + JsonlValueDeserialize + Stream + Send + Sync {
    /// Stream type for the first n lines
    type NLines: Stream<Item = anyhow::Result<String>>;
    /// Stream type for the last n lines (in reverse order)
    type NLinesRev: Stream<Item = anyhow::Result<String>>;

    /// Get the first `n` lines from the JSONL stream.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of lines to retrieve from the beginning
    ///
    /// # Returns
    ///
    /// Returns a stream of the first `n` lines as `String`s, or an error if reading fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_jsonl::{Jsonl, JsonlReader, JsonlDeserialize};
    /// use futures::StreamExt;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct LogEntry {
    ///     timestamp: String,
    ///     level: String,
    ///     message: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let reader = Jsonl::from_path("data.jsonl").await?;
    ///     
    ///     // Get first 3 lines and deserialize them
    ///     let first_three = reader.first_n(3).await?;
    ///     let entries: Vec<LogEntry> = first_three
    ///         .deserialize::<LogEntry>()
    ///         .collect::<Vec<_>>()
    ///         .await
    ///         .into_iter()
    ///         .collect::<Result<Vec<_>, _>>()?;
    ///     
    ///     println!("First 3 log entries: {:?}", entries);
    ///     Ok(())
    /// }
    /// ```
    async fn first_n(self, n: usize) -> anyhow::Result<Self::NLines>;

    /// Get the last `n` lines from the JSONL stream.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of lines to retrieve from the end
    ///
    /// # Returns
    ///
    /// Returns a stream of the last `n` lines as `String`s in reverse order,
    /// or an error if reading fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_jsonl::{Jsonl, JsonlReader};
    /// use futures::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let reader = Jsonl::from_path("data.jsonl").await?;
    ///     
    ///     let last_two = reader.last_n(2).await?;
    ///     let mut stream = last_two;
    ///     
    ///     while let Some(result) = stream.next().await {
    ///         match result {
    ///             Ok(line) => println!("Line: {}", line),
    ///             Err(e) => eprintln!("Error: {}", e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    async fn last_n(self, n: usize) -> anyhow::Result<Self::NLinesRev>;

    /// Count the total number of lines in the JSONL stream.
    async fn count(self) -> usize;
}

/// Extension trait to add deserialization capabilities to JSONL readers.
///
/// This trait provides methods to deserialize JSON lines into strongly-typed Rust structures.
/// It works with any type that implements `serde::Deserialize` and processes each line
/// of a JSONL file as a separate JSON object.
///
/// # Examples
///
/// ## Basic Usage with Custom Types
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlDeserialize};
/// use futures::StreamExt;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct User {
///     id: u64,
///     name: String,
///     email: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let data = r#"{"id": 1, "name": "Alice", "email": "alice@example.com"}
/// {"id": 2, "name": "Bob", "email": "bob@example.com"}"#;
///     let reader = Jsonl::new(Cursor::new(data.as_bytes()));
///     
///     let mut user_stream = reader.deserialize::<User>();
///     
///     while let Some(result) = user_stream.next().await {
///         match result {
///             Ok(user) => println!("User: {} ({})", user.name, user.email),
///             Err(e) => eprintln!("Failed to parse user: {}", e),
///         }
///     }
///     
///     Ok(())
/// }
/// ```
///
/// ## Error Handling and Filtering
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlDeserialize};
/// use futures::StreamExt;
/// use serde::Deserialize;
/// use std::io::Cursor;
///
/// #[derive(Deserialize, Debug)]
/// struct Product {
///     name: String,
///     price: f64,
///     #[serde(default)]
///     in_stock: bool,
/// }
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let data = r#"{"name": "Widget A", "price": 19.99, "in_stock": true}
/// {"name": "Widget B", "price": 29.99, "in_stock": false}"#;
///     let reader = Jsonl::new(Cursor::new(data.as_bytes()));
///     
///     // Filter only successful deserializations and in-stock products
///     let in_stock_products: Vec<Product> = reader
///         .deserialize::<Product>()
///         .filter_map(|result| async move {
///             match result {
///                 Ok(product) if product.in_stock => Some(product),
///                 Ok(_) => None, // Out of stock
///                 Err(e) => {
///                     eprintln!("Skipping invalid product: {}", e);
///                     None
///                 }
///             }
///         })
///         .collect()
///         .await;
///     
///     println!("Found {} products in stock", in_stock_products.len());
///     Ok(())
/// }
/// ```
pub trait JsonlDeserialize {
    /// Deserialize JSON lines into the specified type
    fn deserialize<T>(self) -> impl Stream<Item = anyhow::Result<T>>
    where
        T: for<'a> Deserialize<'a>;
}

/// Extension trait specifically for deserializing JSONL to `serde_json::Value` objects.
///
/// This trait provides a convenient method to deserialize JSON lines into generic
/// `serde_json::Value` objects when you don't know the exact structure of the JSON
/// data ahead of time or when working with heterogeneous JSON objects.
///
/// # Examples
///
/// ## Basic Usage with Dynamic JSON
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlValueDeserialize};
/// use futures::StreamExt;
/// use std::io::Cursor;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let data = r#"{"user_id": 123, "action": "login", "timestamp": "2024-01-01T10:00:00Z"}
/// {"user_id": 456, "action": "logout", "timestamp": "2024-01-01T11:00:00Z"}
/// {"user_id": 789, "action": "purchase", "item": "widget", "price": 29.99}"#;
///     
///     let reader = Jsonl::new(Cursor::new(data.as_bytes()));
///     let mut value_stream = reader.deserialize_values();
///     
///     while let Some(result) = value_stream.next().await {
///         match result {
///             Ok(value) => {
///                 println!("Event: {}", value["action"]);
///                 if let Some(price) = value.get("price") {
///                     println!("  Purchase amount: {}", price);
///                 }
///             }
///             Err(e) => eprintln!("Failed to parse JSON: {}", e),
///         }
///     }
///     
///     Ok(())
/// }
/// ```
///
/// ## Processing Mixed JSON Structures
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlValueDeserialize};
/// use futures::StreamExt;
/// use serde_json::Value;
/// use std::io::Cursor;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mixed_data = r#"{"type": "user", "name": "Alice", "age": 30}
/// {"type": "product", "name": "Widget", "price": 19.99, "categories": ["tools", "hardware"]}
/// {"type": "event", "name": "click", "target": "button", "metadata": {"page": "/home"}}"#;
///     
///     let reader = Jsonl::new(Cursor::new(mixed_data.as_bytes()));
///     let values: Vec<Value> = reader
///         .deserialize_values()
///         .collect::<Vec<_>>()
///         .await
///         .into_iter()
///         .collect::<Result<Vec<_>, _>>()?;
///     
///     for value in values {
///         match value["type"].as_str() {
///             Some("user") => println!("User: {} (age {})", value["name"], value["age"]),
///             Some("product") => println!("Product: {} - ${}", value["name"], value["price"]),
///             Some("event") => println!("Event: {} on {}", value["name"], value["target"]),
///             _ => println!("Unknown type: {:?}", value),
///         }
///     }
///     
///     Ok(())
/// }
/// ```
///
/// ## Error Handling with Invalid JSON
///
/// ```ignore
/// use async_jsonl::{Jsonl, JsonlValueDeserialize};
/// use futures::StreamExt;
/// use std::io::Cursor;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let data_with_errors = r#"{"valid": "json"}
/// {invalid json line
/// {"another": "valid line"}"#;
///     
///     let reader = Jsonl::new(Cursor::new(data_with_errors.as_bytes()));
///     let mut value_stream = reader.deserialize_values();
///     
///     let mut valid_count = 0;
///     let mut error_count = 0;
///     
///     while let Some(result) = value_stream.next().await {
///         match result {
///             Ok(_) => valid_count += 1,
///             Err(_) => error_count += 1,
///         }
///     }
///     
///     println!("Valid JSON lines: {}, Errors: {}", valid_count, error_count);
///     Ok(())
/// }
/// ```
pub trait JsonlValueDeserialize {
    /// Deserialize JSON lines into `serde_json::Value` objects.
    ///
    /// This method transforms each line of a JSONL stream into `serde_json::Value` objects,
    /// which can represent any valid JSON structure. This is useful when:
    ///
    /// - You don't know the exact structure of the JSON data ahead of time
    /// - You're working with heterogeneous JSON objects in the same file
    /// - You want to inspect or transform JSON data dynamically
    /// - You need to handle mixed or evolving JSON schemas
    ///
    /// # Returns
    ///
    /// Returns a `Stream` of `anyhow::Result<Value>` where:
    /// - `Ok(Value)` represents a successfully parsed JSON value
    /// - `Err(anyhow::Error)` represents parsing errors for invalid JSON lines
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use async_jsonl::{Jsonl, JsonlValueDeserialize};
    /// use futures::StreamExt;
    /// use std::io::Cursor;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let data = r#"{"id": 1, "data": {"nested": [1, 2, 3]}}
    /// {"id": 2, "data": {"different": "structure"}}"#;
    ///     
    ///     let reader = Jsonl::new(Cursor::new(data.as_bytes()));
    ///     let values: Vec<_> = reader
    ///         .deserialize_values()
    ///         .collect()
    ///         .await;
    ///     
    ///     for (i, result) in values.iter().enumerate() {
    ///         match result {
    ///             Ok(value) => println!("Object {}: ID = {}", i + 1, value["id"]),
    ///             Err(e) => eprintln!("Error parsing object {}: {}", i + 1, e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    fn deserialize_values(self) -> impl Stream<Item = anyhow::Result<Value>>;
}
