//! Reading architecture parameters out of a GGUF header.
//!
//! Split from `weights.rs` because that file is about moving tensors onto a
//! device and this is about the numbers that describe them. They change for
//! different reasons: a new quantization touches the upload, a new architecture
//! touches these keys.
//!
//! **Every parameter is read, never assumed.** Layer count, head count, rotary
//! base and layer-norm epsilon all vary between models that are otherwise the
//! same family, and a wrong value produces finite, correctly-shaped, wrong
//! vectors — the failure mode with no symptom.

use crate::sys;
use crate::weights::Weights;
use std::ffi::CString;

impl Weights {
    /// A `u32` from the header, or `None` if the key is absent.
    ///
    /// Architecture parameters — layer count, head count, context length — are
    /// read from the file rather than assumed, so a model with a different
    /// shape fails to load instead of producing wrong vectors.
    pub fn u32_meta(&self, key: &str) -> Option<u32> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` was returned by `find_key` for it.
        Some(unsafe { sys::gguf_get_val_u32(self.gguf(), id) })
    }

    /// An `f32` from the header — layer-norm epsilon, rotary base.
    pub fn f32_meta(&self, key: &str) -> Option<f32> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` was returned by `find_key` for it.
        Some(unsafe { sys::gguf_get_val_f32(self.gguf(), id) })
    }

    /// Resolve a metadata key to its index.
    fn key_id(&self, key: &str) -> Option<i64> {
        let key = CString::new(key).ok()?;
        // SAFETY: `gguf` is live; `key` is NUL-terminated and outlives the call.
        let id = unsafe { sys::gguf_find_key(self.gguf(), key.as_ptr()) };
        (id >= 0).then_some(id)
    }
}
