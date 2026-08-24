//! A hard ceiling on how much device memory resident indexes may hold.
//!
//! **Why this exists rather than relying on allocation failure.** Measured on
//! Metal: a corpus that exceeds available memory does not return an error, it
//! *kills the process* — candle aborts rather than surfacing a `Result`, so
//! there is nothing to catch. A budget checked before the upload is the only
//! place the failure can be made recoverable. This is a first policy for
//! ARCHITECTURE §15 Gap 22, not a solution to it.
//!
//! **The accounting is shared, not per index.** Reservations go through
//! [`telividb_telemetry::residency`], so an index competes for the ceiling with
//! every *other* device-resident thing — including models, once the inference
//! server lands. That is what a model-zoo budget has to mean. It also covers
//! the case that actually died: a *rebuild*, holding the old corpus and the new
//! one at once, where two individually-legal indexes together exceeded the
//! device.
//!
//! **Where the ceiling comes from.** [`DEFAULT_FRACTION`] of whatever memory is
//! detected. On Metal that detection is Apple's own
//! `recommendedMaxWorkingSetSize` — what the GPU can use with good performance
//! — which is a real device figure rather than a guess. Elsewhere it falls back
//! to system memory, which is right for the CPU path and **overestimates a
//! discrete CUDA card**, since candle exposes no device-memory API for one. Set
//! `TELIVIDB_GPU_BUDGET_BYTES` explicitly on such a machine.
//!
//! The fraction applies to *both* readings deliberately. Apple's figure is what
//! the GPU can address, not what is free — the rest of the system, and the host
//! copy every upload is made from, still have to fit alongside.

use super::detect::{metal_working_set_size, system_memory};
use std::sync::OnceLock;
use telividb_core::{Error, Result};
use telividb_telemetry::residency::{self, Location, ResidentKind};

/// Fraction of detected memory that resident indexes may occupy.
///
/// Applied to whichever reading is available, device-reported or estimated.
/// Deliberately conservative: the measured failure needed roughly twice one
/// corpus (build plus rebuild), so a third leaves room for both — and Apple's
/// working-set figure describes what the GPU *can* address, not what is
/// currently free.
pub const DEFAULT_FRACTION: f64 = 0.30;

/// A rebuild holds two copies at once, so anything at or above one half could
/// admit a corpus whose own update then exceeds the device. Checked here
/// because it is a property of the constant, not of any particular run.
const _: () = assert!(DEFAULT_FRACTION < 0.5);

/// Environment override, in bytes. Escape hatch for a discrete GPU, where
/// detection overestimates, and for deliberately running closer to the edge.
pub const BUDGET_ENV: &str = "TELIVIDB_GPU_BUDGET_BYTES";

/// Where the ceiling came from, so an operator can tell a measurement from a
/// guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetSource {
    /// Set explicitly through [`BUDGET_ENV`].
    Configured,
    /// Reported by Metal as its recommended working-set size.
    MetalReported,
    /// A fraction of system memory — an estimate, not a device figure.
    Estimated,
}

impl BudgetSource {
    /// The name used in telemetry and operator output.
    pub fn as_str(self) -> &'static str {
        match self {
            BudgetSource::Configured => "configured",
            BudgetSource::MetalReported => "metal-reported",
            BudgetSource::Estimated => "estimated",
        }
    }
}

/// The ceiling and where it came from, computed once.
fn budget() -> (usize, BudgetSource) {
    static BUDGET: OnceLock<(usize, BudgetSource)> = OnceLock::new();
    *BUDGET.get_or_init(|| {
        if let Ok(raw) = std::env::var(BUDGET_ENV)
            && let Ok(bytes) = raw.parse::<usize>()
        {
            return (bytes, BudgetSource::Configured);
        }
        if let Some(bytes) = metal_working_set_size() {
            return (
                (bytes as f64 * DEFAULT_FRACTION) as usize,
                BudgetSource::MetalReported,
            );
        }
        // With no reading at all, assume a modest 4 GiB rather than falling
        // back to unlimited: an undetectable platform is exactly where an
        // unguarded upload would abort the process, so the safe direction is a
        // small budget that refuses, not a large one that crashes.
        let total = system_memory().unwrap_or(4 * 1024 * 1024 * 1024);
        (
            (total as f64 * DEFAULT_FRACTION) as usize,
            BudgetSource::Estimated,
        )
    })
}

/// Claim `bytes` for a device-resident index named `name`, or refuse.
///
/// Refuses rather than evicting: with no policy for *which* index to evict —
/// Gap 22's actual open question — silently dropping someone else's resident
/// corpus would be a worse failure than a clear error here.
pub fn reserve(name: &str, bytes: usize) -> Result<residency::Handle> {
    let (limit, source) = budget();
    let used = residency::total_bytes(Location::Device);

    if used + bytes > limit {
        return Err(Error::GpuIndex {
            reason: format!(
                "index needs {:.1} MiB but only {:.1} MiB of the {:.1} MiB budget is free \
                 (budget {}). Set {BUDGET_ENV} to raise it.",
                mib(bytes),
                mib(limit.saturating_sub(used)),
                mib(limit),
                source.as_str(),
            ),
        });
    }
    Ok(residency::register(
        ResidentKind::VectorIndex,
        Location::Device,
        name,
        bytes,
    ))
}

/// Bytes currently reserved across every device-resident thing.
pub fn resident_bytes() -> usize {
    residency::total_bytes(Location::Device)
}

/// The ceiling in force.
pub fn limit_bytes() -> usize {
    budget().0
}

pub use super::detect::device_allocated_bytes;

/// Where the ceiling came from.
pub fn budget_source() -> BudgetSource {
    budget().1
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

#[cfg(test)]
#[path = "budget_test.rs"]
mod tests;
