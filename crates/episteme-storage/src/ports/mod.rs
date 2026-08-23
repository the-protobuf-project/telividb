//! Boundaries of the storage layer.

mod block_reader;
mod file_reader;

pub use block_reader::BlockReader;
pub use file_reader::{FileBlockReader, MemoryBlockReader};
