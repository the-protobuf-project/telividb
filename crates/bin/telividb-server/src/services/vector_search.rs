//! Searching a vector field, across its sealed segments and live buffer.
//!
//! Split from `vectors.rs` so that file is about *holding* field state and
//! this one about querying it — the write path and the read path have almost
//! nothing in common beyond the map they share.

use super::vectors::VectorFields;
use telividb_core::{Dim, ResourceName, Result};
use telividb_index::VectorIndex;
use telividb_index::adapters::{FlatIndex, GpuFlatIndex};

impl VectorFields {
    /// Search `field` of `collection`, returning `(row, score)` best first.
    ///
    /// Rows are field-wide, so the caller resolves them to resource names —
    /// an ordinal must never leave this process (invariant 9).
    pub fn search(
        &self,
        collection: &ResourceName,
        field: &str,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(usize, f32)>> {
        let mut fields = self.lock();
        let key = (collection.as_str().to_owned(), field.to_owned());

        // Open on first *search*, not only on first append. A restarted
        // process has an empty map but a full WAL on disk, and returning early
        // here would report a durable vector as missing — the bug this
        // replaced, which no unit test could catch because it only appears
        // across a process boundary.
        let state = match fields.entry(key) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let dim = Dim::new(query.len() as u32)?;
                if !self.dir_for(collection, field).exists() {
                    // Never written, here or in an earlier process: an empty
                    // result, not an error. A collection may simply not carry
                    // this field.
                    return Ok(Vec::new());
                }
                e.insert(self.open_field(collection, field, dim)?)
            }
        };

        // Sealed segments and the unsealed buffer are separate stores, so each
        // is searched and the results merged. The buffer's inclusion is what
        // makes a just-written point findable.
        let stores = state.field.stores();
        let mut hits: Vec<(usize, f32)> = Vec::new();
        let mut higher_is_nearer = true;

        for (index, store) in stores.iter().enumerate() {
            if store.is_empty() {
                continue;
            }
            higher_is_nearer = store.metric().higher_is_nearer();

            // The GPU index is built over the *whole* field only when it is a
            // single store; with several, the flat index scores each without a
            // separate upload per segment. Rebuilding a device corpus per
            // segment per query would cost far more than it saves.
            let found = if stores.len() == 1 {
                let gpu = match state.index.take() {
                    Some(gpu) => gpu,
                    None => GpuFlatIndex::build(*store)?,
                };
                let found = gpu.search(*store, query, k, None)?;
                state.index = Some(gpu);
                found
            } else {
                FlatIndex::new().search(*store, query, k, None)?
            };

            for candidate in found {
                hits.push((
                    state.field.row_of(index, candidate.ordinal),
                    candidate.score,
                ));
            }
        }

        // One ordering across every store, then truncate — the merge step
        // ARCHITECTURE §4.1 requires before top-k.
        hits.sort_by(|a, b| {
            if higher_is_nearer {
                b.1.total_cmp(&a.1)
            } else {
                a.1.total_cmp(&b.1)
            }
        });
        hits.truncate(k);
        Ok(hits)
    }
}
