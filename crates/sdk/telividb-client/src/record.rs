//! A point, read back.

use crate::convert;
use crate::error::Result;
use crate::names;
use std::collections::BTreeMap;
use telividb_buffers::protobuf::point::v1 as wire;

/// A stored point.
///
/// A type of this crate's own rather than the generated `Point`, so a caller
/// works with `Vec<f32>` and `String` instead of `bytes` fields and optional
/// nested messages — and so the wire shape can change without every caller's
/// code changing with it.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The point's id within its collection.
    pub name: String,

    /// Text the point carries inline, if any.
    pub text: Option<String>,

    /// Vectors by field name.
    ///
    /// **Empty today, and that is deliberate rather than missing.** Vectors
    /// live in the columnar field, not in the point's metadata record, and
    /// reading raw vectors back is its own permission scope (`vector.<field>`,
    /// invariant 12's neighbour) — not something a plain `GetPoint` should
    /// hand out. When the server grows a `read_mask` path for it, a caller who
    /// holds that grant will see the field populated here with no change to
    /// this type.
    ///
    /// Search does not need this: a hit carries a score and the point's
    /// identity, which is what ranking is for.
    ///
    /// A `BTreeMap` so iteration order is stable: a caller printing or hashing
    /// this should get the same result twice, which a `HashMap` would not
    /// promise. A field being absent is ordinary, not an error — a point
    /// carries whichever named fields were written to it, never all of them
    /// (invariant 17).
    pub vectors: BTreeMap<String, Vec<f32>>,
}

impl Record {
    /// Read a point off the wire.
    pub(crate) fn from_wire(point: wire::Point) -> Result<Self> {
        let mut vectors = BTreeMap::new();
        for named in &point.vectors {
            if let Some(vector) = &named.vector {
                vectors.insert(named.field_id.clone(), convert::from_wire(vector)?);
            }
        }

        Ok(Self {
            name: names::id_of(&point.name).to_owned(),
            text: convert::inline_text(&point),
            vectors,
        })
    }
}
