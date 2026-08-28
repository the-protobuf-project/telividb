//! Carrying a runtime failure into this crate's error type.
//!
//! A `From` impl is out of reach in both directions: `telividb_core::Error` and
//! `telividb_compute::Error` are each defined elsewhere, so the orphan rule
//! rules out implementing the conversion here. An extension trait on the
//! runtime's `Result` gives the same ergonomics — `.on_device()?` — without
//! either crate having to know about the other.
//!
//! It also keeps the direction right: `telividb-core` must not gain a
//! dependency on a tensor runtime just to name its errors.

use telividb_core::Result;

/// Turning a [`telividb_compute`] result into one this crate returns.
pub(super) trait OnDevice<T> {
    /// Re-tag the error as a GPU index failure, preserving its message.
    fn on_device(self) -> Result<T>;
}

impl<T> OnDevice<T> for telividb_compute::Result<T> {
    fn on_device(self) -> Result<T> {
        self.map_err(|e| telividb_core::Error::GpuIndex {
            reason: e.to_string(),
        })
    }
}
