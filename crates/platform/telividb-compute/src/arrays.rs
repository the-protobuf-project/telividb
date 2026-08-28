//! Array-valued GGUF metadata.
//!
//! Separate from the scalar accessors because the two fail differently: a
//! scalar is present or absent, while an array can be present with the wrong
//! element type — and reading an `i32` array as `i8` succeeds, returning
//! plausible values that are wrong. Every reader here checks the declared
//! element type before touching the bytes.
//!
//! What needs this is the tokenizer: `tokenizer.ggml.tokens` is the vocabulary
//! and `tokenizer.ggml.token_type` says which entries are special. Reading them
//! from the model file rather than a sidecar is what makes rule 12 hold — swap
//! the tokenizer and the digest changes with it.

use crate::header::Header;
use crate::sys;
use std::ffi::CStr;

impl Header {
    /// A string array — the vocabulary, above all.
    pub fn str_array(&self, key: &str) -> Option<Vec<String>> {
        let id = self.array_id(key, sys::gguf_type_GGUF_TYPE_STRING)?;
        // SAFETY: `id` names a live array of declared type STRING, so every
        // index below `len` yields a NUL-terminated pointer owned by the gguf
        // context and valid while it is.
        unsafe {
            let len = sys::gguf_get_arr_n(self.raw(), id);
            Some(
                (0..len)
                    .map(|i| {
                        let raw = sys::gguf_get_arr_str(self.raw(), id, i);
                        match raw.is_null() {
                            true => String::new(),
                            false => CStr::from_ptr(raw).to_string_lossy().into_owned(),
                        }
                    })
                    .collect(),
            )
        }
    }

    /// An `i32` array — per-token type codes.
    ///
    /// Converters have written these as either signed or unsigned at different
    /// times, so both are accepted; the bit patterns are identical for the
    /// small values actually used.
    pub fn i32_array(&self, key: &str) -> Option<Vec<i32>> {
        let id = self
            .array_id(key, sys::gguf_type_GGUF_TYPE_INT32)
            .or_else(|| self.array_id(key, sys::gguf_type_GGUF_TYPE_UINT32))?;

        // SAFETY: the declared element type is a 32-bit integer and `len` is
        // the element count ggml recorded, so the slice is exactly the array.
        unsafe {
            let len = sys::gguf_get_arr_n(self.raw(), id);
            let data = sys::gguf_get_arr_data(self.raw(), id).cast::<i32>();
            match data.is_null() {
                true => None,
                false => Some(std::slice::from_raw_parts(data, len).to_vec()),
            }
        }
    }

    /// Resolve `key` to an array of the expected element type.
    ///
    /// Returns `None` when the key is absent, is not an array, or holds a
    /// different element type — the last of which is the case worth checking,
    /// because reading past it succeeds and returns nonsense.
    fn array_id(&self, key: &str, element: sys::gguf_type) -> Option<i64> {
        let key = std::ffi::CString::new(key).ok()?;
        // SAFETY: `gguf` is live; `key` is NUL-terminated and outlives the call.
        unsafe {
            let id = sys::gguf_find_key(self.raw(), key.as_ptr());
            if id < 0 || sys::gguf_get_kv_type(self.raw(), id) != sys::gguf_type_GGUF_TYPE_ARRAY {
                return None;
            }
            (sys::gguf_get_arr_type(self.raw(), id) == element).then_some(id)
        }
    }
}
