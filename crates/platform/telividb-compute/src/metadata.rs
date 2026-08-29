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

use crate::header::Header;
use crate::sys;
use std::ffi::CString;

impl Header {
    /// A `u32` from the header, or `None` if the key is absent.
    ///
    /// Architecture parameters — layer count, head count, context length — are
    /// read from the file rather than assumed, so a model with a different
    /// shape fails to load instead of producing wrong vectors.
    pub fn u32_meta(&self, key: &str) -> Option<u32> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` was returned by `find_key` for it.
        Some(unsafe { sys::gguf_get_val_u32(self.raw(), id) })
    }

    /// A `bool` from the header.
    ///
    /// Sparse in practice, and every use of it so far is a tokenizer flag —
    /// `add_bos_token`, `add_eos_token` — which decide whether a sequence is
    /// terminated. That matters more than it sounds: a model pooling its last
    /// token reads the position *after* the text, so a missing terminator
    /// silently pools the final word instead of the summary state.
    pub fn bool_meta(&self, key: &str) -> Option<bool> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` was returned by `find_key` for it.
        Some(unsafe { sys::gguf_get_val_bool(self.raw(), id) })
    }

    /// An `f32` from the header — layer-norm epsilon, rotary base.
    pub fn f32_meta(&self, key: &str) -> Option<f32> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` was returned by `find_key` for it.
        Some(unsafe { sys::gguf_get_val_f32(self.raw(), id) })
    }

    /// A string from the header — `general.architecture`, above all.
    ///
    /// The architecture name is the prefix every other key hangs off, so it is
    /// read first and everything else is derived from it. Guessing it wrong
    /// finds no tensors at all, which is the failure worth having: loud.
    pub fn str_meta(&self, key: &str) -> Option<String> {
        let id = self.key_id(key)?;
        // SAFETY: `gguf` is live and `id` came from `find_key` on it. The
        // returned pointer is owned by the gguf context and valid while it is.
        let raw = unsafe { sys::gguf_get_val_str(self.raw(), id) };
        match raw.is_null() {
            true => None,
            // SAFETY: non-null and NUL-terminated by ggml's own contract.
            false => Some(
                unsafe { std::ffi::CStr::from_ptr(raw) }
                    .to_string_lossy()
                    .into_owned(),
            ),
        }
    }

    /// Resolve a metadata key to its index.
    fn key_id(&self, key: &str) -> Option<i64> {
        let key = CString::new(key).ok()?;
        // SAFETY: `gguf` is live; `key` is NUL-terminated and outlives the call.
        let id = unsafe { sys::gguf_find_key(self.raw(), key.as_ptr()) };
        (id >= 0).then_some(id)
    }
}
