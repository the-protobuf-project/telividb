//! A store's rows, uploaded to a device and kept there.
//!
//! Thin over [`telividb_compute::Corpus`], which owns the device memory and the
//! backend. What this adds is the part the runtime has no opinion about: which
//! rows are actually present, what metric their scores mean, and the row norms
//! the L2 expansion needs.
//!
//! **Presence is tracked, not inferred.** An absent row is uploaded as zeros
//! because the corpus is a dense matrix and must stay one. Against a dot
//! product zero is a *real* score rather than a neutral one — it beats every
//! negative score — so an absent row would otherwise rank. The bitmap is what
//! keeps it out of selection.

use super::error::OnDevice;
use super::scored::Scored;
use std::sync::Mutex;
use telividb_compute::{Backend, Corpus};
use telividb_core::{Dim, Metric, Ordinal, Result, VectorStore};

/// Rows staged in host memory at once, bounding the copy that upload needs.
///
/// 64k rows is 32 MB at 128 dimensions — large enough that per-write overhead
/// is irrelevant, small enough to stay in cache-friendly territory and to be
/// independent of corpus size, which is the whole point.
const CHUNK_ROWS: usize = 65_536;

/// Vectors resident on a device, with what is needed to score them.
pub(super) struct DeviceCorpus {
    /// The device-resident matrix. Owns the backend and the device memory.
    ///
    /// **Behind a lock because a `ggml` backend is not safe to compute on from
    /// two threads at once** — it holds one command queue, and concurrent
    /// submissions to it race. The lock is held for the device call only and
    /// released before selection, so the CPU-bound half of a search still runs
    /// concurrently across callers.
    ///
    /// This costs less than it appears to: a GPU executes submitted work
    /// serially regardless, so the lock reorders nothing that was actually
    /// parallel. candle's Metal device holds an equivalent internal mutex for
    /// the same reason — the difference is that here it is visible.
    ///
    /// `None` for an empty field, which is entirely ordinary — a collection
    /// with no points yet, or a field none of them populate. Nothing is
    /// uploaded in that case rather than a zero-row matrix being forced into
    /// existence: a tensor with no elements is a shape `ggml` rejects, and
    /// there is nothing to score against anyway.
    inner: Option<Mutex<Corpus>>,
    /// How many rows are resident, cached so the common questions — is the
    /// corpus empty, how many rows does it have — never take the lock.
    rows: usize,
    /// Which rows hold a real vector, by ordinal. See the module note.
    pub(super) present: Vec<bool>,
    /// `‖row‖²` for every row, for the L2 expansion.
    ///
    /// Computed once on the host while the rows are already in hand, rather
    /// than as a device reduction afterwards: they depend only on the stored
    /// vectors, so the alternative is a second pass over the whole corpus to
    /// recover numbers that were free at upload time. Empty for metrics that
    /// do not need them, since it would be a wasted allocation per row.
    pub(super) row_norms: Vec<f32>,
    /// The width every vector shares.
    pub(super) dim: Dim,
    /// What a score means, and therefore which direction is nearer.
    pub(super) metric: Metric,
}

impl DeviceCorpus {
    /// Copy every present row of `store` onto `backend`.
    pub(super) fn from_store(store: &dyn VectorStore, backend: Backend) -> Result<Self> {
        let dim = store.dim();
        let rows = store.len();
        let metric = store.metric();
        let width = dim.get();

        // Staged in bounded pieces rather than materialized whole. A full host
        // mirror is a second copy of the entire corpus — 512 MB at a million
        // 128-dimension rows — allocated and zeroed before the first useful
        // byte is written. Measured, that made build time superlinear in rows:
        // 0.168 s at 500k against 1.681 s at 4M, and 14.8 s under a harness
        // holding the source dataset alongside.
        // `None` for an empty field, which is entirely ordinary — a collection
        // with no points yet, or a field none of them populate. `ggml` has no
        // zero-element tensor, and there is nothing to score against anyway.
        let mut staged = match rows {
            0 => None,
            _ => Some(Corpus::staged(backend, rows, width).on_device()?),
        };
        let mut chunk: Vec<f32> = Vec::with_capacity(CHUNK_ROWS * width);
        let mut present = Vec::with_capacity(rows);
        let mut row_norms = match metric {
            Metric::L2 => Vec::with_capacity(rows),
            _ => Vec::new(),
        };

        for row in 0..rows {
            let vector = store.get(Ordinal::from_row(row as u32));
            match vector {
                Some(vector) => chunk.extend_from_slice(vector),
                // An absent row is uploaded as zeros: the corpus is a dense
                // matrix and must stay one. The presence bitmap is what keeps
                // it out of selection.
                None => chunk.extend(std::iter::repeat_n(0.0, width)),
            }
            present.push(vector.is_some());

            if chunk.len() == chunk.capacity()
                && let Some(staged) = staged.as_mut()
            {
                staged.push_rows(&chunk).on_device()?;
                chunk.clear();
            }

            // While the row is in hand: recovering this later means reading the
            // corpus a second time, on the device, to compute what a single
            // multiply-add per element gives for nothing here.
            if metric == Metric::L2 {
                row_norms.push(vector.map_or(0.0, |v| v.iter().map(|x| x * x).sum()));
            }
        }

        let inner = match staged {
            None => None,
            Some(mut staged) => {
                if !chunk.is_empty() {
                    staged.push_rows(&chunk).on_device()?;
                }
                Some(Mutex::new(staged.finish().on_device()?))
            }
        };

        Ok(Self {
            inner,
            rows,
            present,
            row_norms,
            dim,
            metric,
        })
    }

    /// Bytes a corpus built from `store` will occupy on the device.
    ///
    /// Reserved *before* the upload, because an over-large allocation does not
    /// fail gracefully on Metal — it aborts the process — so this is the last
    /// point at which the failure is still recoverable.
    pub(super) fn device_bytes(store: &dyn VectorStore) -> usize {
        store
            .len()
            .saturating_mul(store.dim().get())
            .saturating_mul(std::mem::size_of::<f32>())
    }

    /// How many rows are resident, present or not.
    pub(super) fn rows(&self) -> usize {
        self.rows
    }

    /// Score `queries` — `count` of them, laid end to end — against every row.
    ///
    /// Returns the metric's scores, not raw inner products: the device computes
    /// the product, and [`ScoreFromDot`] turns it into a distance where the
    /// metric asks for one.
    pub(super) fn score(&self, queries: &[f32], count: usize) -> Result<Scored> {
        let Some(inner) = self.inner.as_ref() else {
            // No rows to score against. The callers already short-circuit on an
            // empty corpus, so this is a second line rather than the first.
            return Ok(Scored {
                scores: telividb_compute::Scores::empty(count),
                query_norms: Vec::new(),
            });
        };

        let scores = inner
            .lock()
            .map_err(|_| telividb_core::Error::GpuIndex {
                reason: "the device corpus lock was poisoned by an earlier panic".to_owned(),
            })?
            .score_batch(queries, count)
            .on_device()?;

        Ok(Scored {
            scores,
            query_norms: match self.metric {
                Metric::L2 => queries
                    .chunks_exact(self.dim.get())
                    .map(|q| q.iter().map(|x| x * x).sum())
                    .collect(),
                _ => Vec::new(),
            },
        })
    }
}
