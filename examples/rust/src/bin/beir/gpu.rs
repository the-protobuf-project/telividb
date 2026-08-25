//! Watching what the GPU actually holds.
//!
//! **Why two numbers and not one.** The residency registry says what this
//! process *believes* it reserved; Metal's `currentAllocatedSize` says what
//! the driver actually holds. A leak is precisely the case where those two
//! disagree over time — memory allocated that nothing reserved, or a
//! reservation whose memory was never returned. One number alone cannot show
//! it: the registry would look tidy while the device filled up.

use telividb_index::adapters::device_allocated_bytes;
use telividb_telemetry::residency::{Location, ResidentKind, count, snapshot, total_bytes};

/// One observation of device memory.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    /// Bytes this process reserved through the residency registry.
    pub reserved: usize,
    /// Bytes Metal reports allocated device-wide, when it reports any.
    ///
    /// `None` off Metal — CUDA exposes no equivalent through candle, and a CPU
    /// fallback has no device memory to report.
    pub allocated: Option<usize>,
    /// Models resident. More than one after a single-model run means a
    /// handle outlived its owner.
    pub models: usize,
    /// Resident vector indexes.
    pub indexes: usize,
}

/// Take a reading.
pub fn sample() -> Sample {
    Sample {
        reserved: total_bytes(Location::Device),
        allocated: device_allocated_bytes(),
        models: count(ResidentKind::Model),
        indexes: count(ResidentKind::VectorIndex),
    }
}

impl Sample {
    /// Bytes that grew between two readings, as a signed figure.
    pub fn growth_since(&self, earlier: &Sample) -> (i64, Option<i64>) {
        (
            self.reserved as i64 - earlier.reserved as i64,
            match (self.allocated, earlier.allocated) {
                (Some(now), Some(before)) => Some(now as i64 - before as i64),
                _ => None,
            },
        )
    }
}

/// Everything currently resident, by name.
///
/// Real names rather than redacted ones: rule 28 governs what reaches a
/// telemetry pipeline, and this is an operator asking what their own process
/// holds. A table of opaque tokens would answer nothing.
pub fn resident_lines() -> Vec<String> {
    snapshot()
        .into_iter()
        .map(|entry| {
            format!(
                "{:<12} {:<7} {:>9.1} MiB  {}",
                entry.kind.as_str(),
                entry.location.as_str(),
                entry.bytes as f64 / (1024.0 * 1024.0),
                entry.name,
            )
        })
        .collect()
}

/// Bytes as MiB, for a table.
pub fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Signed bytes as MiB.
pub fn mib_signed(bytes: i64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
