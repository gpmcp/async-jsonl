mod buf_reader;
mod lines;

pub use buf_reader::RevBufReader;
pub use lines::Lines;

const DEFAULT_BUF_SIZE: usize = 8 * 1024;
