//! Where bytes sit inside a sealed segment.

use std::path::{Path, PathBuf};

/// Directory holding one named vector field's files.
///
/// Per-field rather than per-segment because a point carries several vector
/// fields with different dimensions, models and codecs — see the multimodal
/// data model. Each gets its own `raw.bin`, index and presence bitmap.
pub fn field_dir(segment: &Path, field: &str) -> PathBuf {
    segment.join("vectors").join(field)
}

/// Byte offsets within one field's `raw.bin`.
///
/// The layout is a header followed by fixed-stride rows, aligned so that a
/// mapped region casts straight to a float slice with no copy. That alignment
/// is why the header is padded rather than packed tight against the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldLayout {
    pub row_bytes: usize,
    pub rows: u64,
    /// Where row zero begins. Aligned to [`FieldLayout::ALIGN`].
    pub data_offset: u64,
}

impl FieldLayout {
    /// Rows begin on a 64-byte boundary so a mapped region feeds SIMD directly.
    pub const ALIGN: u64 = 64;

    pub fn new(header_bytes: usize, row_bytes: usize, rows: u64) -> Self {
        Self {
            row_bytes,
            rows,
            data_offset: align_up(header_bytes as u64, Self::ALIGN),
        }
    }

    /// Offset of one row.
    pub fn row_offset(&self, row: u64) -> u64 {
        self.data_offset + row * self.row_bytes as u64
    }

    /// Total file size.
    pub fn total_bytes(&self) -> u64 {
        self.data_offset + self.rows * self.row_bytes as u64
    }

    /// Bytes of zero padding between header and data.
    pub fn padding(&self, header_bytes: usize) -> usize {
        (self.data_offset - header_bytes as u64) as usize
    }
}

/// Round `value` up to the next multiple of `align`.
pub fn align_up(value: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "alignment must be a power of two");
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
#[path = "layout_test.rs"]
mod tests;
