//! Filling a device corpus in bounded pieces.
//!
//! [`Corpus::upload`] takes the whole corpus as one host slice, which means the
//! caller holds a second full copy of it in host memory while the upload runs.
//! At a million 128-dimension rows that is 512 MB of staging beside the 512 MB
//! already in the store — and it is *allocated and zeroed* before the first
//! useful byte is written, so the cost is paid whether or not the pages are
//! ever read.
//!
//! Measured, the effect is superlinear rather than merely additive: build time
//! went 0.168 s at 500k rows to 1.681 s at 4M — four times the rows for eight
//! times the time — because the allocation competes with everything else
//! resident. Under a benchmark harness holding the source dataset as well, the
//! same build measured 14.8 s.
//!
//! Writing in chunks bounds host staging to one chunk regardless of corpus
//! size. The device allocation is unchanged; only the host-side mirror shrinks.

use crate::backend::Backend;
use crate::corpus::Corpus;
use crate::error::{Error, Result};
use crate::sys;

/// A device corpus being filled row by row.
///
/// Rows are appended in order and the corpus is only usable once every row has
/// been written — an unfilled tensor holds whatever the allocator left there,
/// which would score as though those rows were real data.
pub struct Staged {
    corpus: Corpus,
    /// Rows written so far, and therefore where the next write lands.
    filled: usize,
}

impl Corpus {
    /// Allocate device memory for `rows * dim` floats, to be filled by
    /// [`Staged::push_rows`].
    ///
    /// Prefer [`Corpus::upload`] when the corpus is already contiguous in host
    /// memory; this exists for the case where materializing it whole is the
    /// expensive part.
    pub fn staged(backend: Backend, rows: usize, dim: usize) -> Result<Staged> {
        Ok(Staged {
            corpus: Self::empty(backend, rows, dim)?,
            filled: 0,
        })
    }
}

impl Staged {
    /// Append `vectors` — a whole number of `dim`-wide rows — to the corpus.
    pub fn push_rows(&mut self, vectors: &[f32]) -> Result<()> {
        let dim = self.corpus.dim();
        if !vectors.len().is_multiple_of(dim) {
            return Err(Error::ShapeMismatch {
                expected: format!("a multiple of {dim} floats"),
                actual: format!("{}", vectors.len()),
            });
        }

        let rows = vectors.len() / dim;
        if self.filled + rows > self.corpus.rows() {
            return Err(Error::ShapeMismatch {
                expected: format!("at most {} rows", self.corpus.rows()),
                actual: format!("{}", self.filled + rows),
            });
        }

        // SAFETY: the tensor is backed by device memory sized `rows * dim`
        // floats, and the bounds check above puts this write entirely inside
        // it. `offset` counts bytes from the start of the tensor, which is
        // where row `filled` begins because rows are contiguous.
        unsafe {
            sys::ggml_backend_tensor_set(
                self.corpus.tensor(),
                vectors.as_ptr().cast(),
                self.filled * dim * std::mem::size_of::<f32>(),
                std::mem::size_of_val(vectors),
            )
        };
        self.filled += rows;
        Ok(())
    }

    /// The finished corpus, once every row has been written.
    ///
    /// Refuses a partially filled one rather than returning it: the unwritten
    /// tail is uninitialized device memory, and scoring against it produces
    /// confident nonsense rather than an error.
    pub fn finish(self) -> Result<Corpus> {
        match self.filled == self.corpus.rows() {
            true => Ok(self.corpus),
            false => Err(Error::ShapeMismatch {
                expected: format!("{} rows written", self.corpus.rows()),
                actual: format!("{}", self.filled),
            }),
        }
    }
}

/// Backends this crate builds against, for a caller choosing one.
impl Staged {
    /// The backend the corpus is being built on.
    pub fn backend(&self) -> &Backend {
        self.corpus.backend()
    }
}
