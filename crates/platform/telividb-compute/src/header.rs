//! A GGUF's metadata, without its tensors.
//!
//! Reading a vocabulary should not cost a model's residency. `gguf_init_from_file`
//! with a null context parses the header alone — key/value pairs and tensor
//! *descriptions*, no tensor data and no device allocation — which is all the
//! tokenizer, the architecture parameters and any inspection tool actually
//! need.
//!
//! [`Weights`] owns one of these and delegates every metadata question to it,
//! so there is one implementation of "what does this key say" whether or not
//! the weights were loaded.
//!
//! [`Weights`]: crate::Weights

use crate::error::{Error, Result};
use crate::sys;
use std::ffi::CString;
use std::path::Path;

/// The parsed key/value section of a GGUF file.
pub struct Header {
    raw: *mut sys::gguf_context,
}

// SAFETY: the pointer is owned by this value and freed exactly once in `Drop`.
// Reads through it are const and do not mutate shared state.
unsafe impl Send for Header {}

impl Header {
    /// Parse `path`'s header, reading no tensor data.
    pub fn open(path: &Path) -> Result<Self> {
        let c_path =
            CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| Error::Runtime {
                op: "gguf_open",
                reason: "the model path contains an interior NUL byte".to_owned(),
            })?;

        // SAFETY: `c_path` is NUL-terminated and outlives the call. A null
        // `ctx` with `no_alloc` is what makes ggml stop after the header.
        let raw = unsafe {
            sys::gguf_init_from_file(
                c_path.as_ptr(),
                sys::gguf_init_params {
                    no_alloc: true,
                    ctx: std::ptr::null_mut(),
                },
            )
        };
        match raw.is_null() {
            true => Err(Error::Runtime {
                op: "gguf_open",
                reason: format!("{} is not a readable GGUF file", path.display()),
            }),
            false => Ok(Self { raw }),
        }
    }

    /// Wrap a context another loader already parsed.
    pub(crate) fn from_raw(raw: *mut sys::gguf_context) -> Self {
        Self { raw }
    }

    /// The underlying context, for the accessor modules.
    pub(crate) fn raw(&self) -> *mut sys::gguf_context {
        self.raw
    }
}

impl Drop for Header {
    fn drop(&mut self) {
        // SAFETY: produced by `open` or handed over by `Weights`, freed once.
        unsafe { sys::gguf_free(self.raw) };
    }
}
