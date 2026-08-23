//! Writing the compressed scan tier.
//!
//! Separate from the segment writer because it answers a different question:
//! the writer decides what a segment *is*, this decides how one field is
//! compressed within it.

use crate::error::Result;
use crate::format::Codec;
use crate::format::quantize::{BinaryCodes, F16Row, Int8Row, PqCodebook, PqParams};
use episteme_core::VectorStore;
use std::path::Path;

/// Write the compressed scan tier.
///
/// Every row occupies its slot whether present or not, so `codes.bin` keeps the
/// same fixed stride as `raw.bin` and a row's offset is computable from its
/// ordinal alone. PQ additionally writes its codebook, because a code is
/// meaningless without exactly the codebook that produced it.
pub(super) fn write_codes(dir: &Path, store: &dyn VectorStore, codec: Codec) -> Result<()> {
    // No codec means no scan tier and no file. The caller already guards this,
    // but the function used to create an empty `codes.bin` before checking —
    // leaving a stray zero-length file inside a sealed, immutable segment.
    if codec == Codec::None {
        return Ok(());
    }
    let dim = store.dim().get();
    let row_bytes = codec.row_bytes(dim);
    let mut out = Vec::with_capacity(store.len() * row_bytes);

    // PQ must see the whole field before it can encode any of it.
    let codebook = if let Codec::Pq { m } = codec {
        let rows: Vec<&[f32]> = (0..store.len())
            .filter_map(|r| store.get(episteme_core::Ordinal::from_row(r as u32)))
            .collect();
        Some(PqCodebook::train(
            &rows,
            dim,
            PqParams {
                m: m as usize,
                ..Default::default()
            },
        )?)
    } else {
        None
    };

    for row in 0..store.len() {
        let ordinal = episteme_core::Ordinal::from_row(row as u32);
        let Some(vector) = store.get(ordinal) else {
            out.extend(std::iter::repeat_n(0u8, row_bytes));
            continue;
        };
        match codec {
            Codec::None => {}
            Codec::F16 => F16Row::encode(vector).write_to(&mut out),
            Codec::Int8 => Int8Row::encode(vector).write_to(&mut out),
            Codec::Binary => out.extend_from_slice(BinaryCodes::encode(vector).as_bytes()),
            Codec::Pq { .. } => {
                let book = codebook.as_ref().expect("trained above for pq");
                out.extend_from_slice(&book.encode(vector)?);
            }
        }
    }

    super::durable::write_synced(dir.join("codes.bin"), &out)?;

    if let Some(book) = codebook {
        let mut bytes = Vec::with_capacity(book.encoded_len());
        book.write_to(&mut bytes);
        // Synced like every other file in the segment: a code is meaningless
        // without exactly the codebook that produced it, so publishing one
        // without the other is worse than publishing neither.
        super::durable::write_synced(dir.join("codebook.pq"), &bytes)?;
    }
    Ok(())
}
