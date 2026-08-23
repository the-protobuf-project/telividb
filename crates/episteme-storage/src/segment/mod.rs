//! Sealing a buffer into an immutable segment, and reading one back.

pub(crate) mod layout;
mod reader;
mod spans;
mod tier_reader;
mod writer;

pub use layout::{FieldLayout, field_dir};
pub use reader::SegmentReader;
pub use spans::{SPAN_BYTES, decode as decode_spans, encode as encode_spans};
pub use tier_reader::open_tier;
pub use writer::SegmentWriter;
