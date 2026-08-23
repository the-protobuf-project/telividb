//! Sealing a buffer into an immutable segment, and reading one back.

mod layout;
mod reader;
mod writer;

pub use layout::{FieldLayout, field_dir};
pub use reader::SegmentReader;
pub use writer::SegmentWriter;
