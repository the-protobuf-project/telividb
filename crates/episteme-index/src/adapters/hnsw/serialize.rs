//! The on-disk form of the graph.
//!
//! Rebuilding on open is fine at ten thousand rows and impossible at a hundred
//! million — an HNSW build is minutes to hours, and a restart cannot pay that.
//!
//! The layout is **flat and offset-addressed**, not a nested structure: a
//! header, then per-node offsets, then one contiguous run of neighbour ids.
//! Reading a node's neighbours is two lookups and a slice, so the file can be
//! used straight from a mapped region with no deserialization pass — which is
//! the whole reason the format is shaped this way rather than being whatever a
//! derive macro would emit.

use super::graph::Graph;
use episteme_core::{Error, Result};

pub const GRAPH_MAGIC: [u8; 4] = *b"EPHN";
pub const GRAPH_VERSION: u16 = 1;

/// `magic(4) version(2) reserved(2) nodes(8) entry(4) max_level(4) edges(8)`
const HEADER_BYTES: usize = 32;

/// Serialize `graph` into a flat, offset-addressed buffer.
pub fn encode(graph: &Graph) -> Vec<u8> {
    let nodes = graph.len();
    let mut out = Vec::with_capacity(HEADER_BYTES + nodes * 8 + graph.edge_count() * 4);

    out.extend_from_slice(&GRAPH_MAGIC);
    out.extend_from_slice(&GRAPH_VERSION.to_le_bytes());
    out.extend_from_slice(&[0u8; 2]);
    out.extend_from_slice(&(nodes as u64).to_le_bytes());
    // `u32::MAX` stands for "no entry", which only an empty graph has.
    out.extend_from_slice(&graph.entry().map_or(u32::MAX, |o| o.row()).to_le_bytes());
    out.extend_from_slice(&(graph.max_level() as u32).to_le_bytes());
    out.extend_from_slice(&(graph.edge_count() as u64).to_le_bytes());

    // Per-node: level, then one length per layer. Written before the edge run
    // so a reader can compute every offset without scanning the edges.
    let mut edges = Vec::with_capacity(graph.edge_count());
    for node in 0..nodes as u32 {
        let level = graph.level_of(node);
        out.extend_from_slice(&(level as u32).to_le_bytes());
        for layer in 0..=level {
            let neighbours = graph.neighbours(node, layer);
            out.extend_from_slice(&(neighbours.len() as u32).to_le_bytes());
            edges.extend_from_slice(neighbours);
        }
    }

    for id in edges {
        out.extend_from_slice(&id.to_le_bytes());
    }
    out
}

/// Rebuild a graph from bytes written by [`encode`].
pub fn decode(bytes: &[u8]) -> Result<Graph> {
    let mut cursor = Cursor::new(bytes);

    let magic = cursor.take(4)?;
    if magic != GRAPH_MAGIC {
        return Err(Error::MalformedIndex {
            reason: "not an episteme hnsw graph",
        });
    }
    if cursor.u16()? > GRAPH_VERSION {
        return Err(Error::MalformedIndex {
            reason: "hnsw graph written by a newer episteme",
        });
    }
    cursor.take(2)?; // reserved

    let nodes = cursor.u64()? as usize;
    let _entry = cursor.u32()?;
    let _max_level = cursor.u32()?;
    let _edges = cursor.u64()?;

    // Never allocate on a length the file claims. The smallest possible node
    // costs eight bytes — a level and one layer length — so a node count that
    // could not fit in what remains is a lie, and reserving on it would abort
    // the process rather than return an error.
    const MIN_NODE_BYTES: usize = 8;
    if nodes.saturating_mul(MIN_NODE_BYTES) > cursor.remaining() {
        return Err(Error::MalformedIndex {
            reason: "node count exceeds the bytes available",
        });
    }

    // Levels and per-layer lengths, in one pass.
    let mut shape: Vec<(usize, Vec<usize>)> = Vec::with_capacity(nodes);
    for _ in 0..nodes {
        let level = cursor.u32()? as usize;
        // Same rule one level down: a layer count must fit in what is left.
        if level.saturating_mul(4) > cursor.remaining() {
            return Err(Error::MalformedIndex {
                reason: "node level exceeds the bytes available",
            });
        }
        let mut lengths = Vec::with_capacity(level + 1);
        for _ in 0..=level {
            lengths.push(cursor.u32()? as usize);
        }
        shape.push((level, lengths));
    }

    let mut graph = Graph::new();
    for (level, _) in &shape {
        graph.push_node(*level);
    }
    for (node, (_, lengths)) in shape.iter().enumerate() {
        for (layer, &count) in lengths.iter().enumerate() {
            if count.saturating_mul(4) > cursor.remaining() {
                return Err(Error::MalformedIndex {
                    reason: "neighbour count exceeds the bytes available",
                });
            }
            let mut neighbours = Vec::with_capacity(count);
            for _ in 0..count {
                neighbours.push(cursor.u32()?);
            }
            graph.set_neighbours(node as u32, layer, neighbours);
        }
    }
    Ok(graph)
}

/// Bounds-checked sequential reader.
///
/// Every read is checked because a graph file is untrusted input the moment an
/// archive arrives from elsewhere — a length field lying about its size must
/// produce an error, never a panic or an out-of-bounds read.
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// Bytes not yet consumed. Every length read from the file is checked
    /// against this before it is used to allocate.
    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or(Error::MalformedIndex {
            reason: "length overflow",
        })?;
        let slice = self.bytes.get(self.at..end).ok_or(Error::MalformedIndex {
            reason: "truncated",
        })?;
        self.at = end;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("2 bytes"),
        ))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 bytes"),
        ))
    }
}

#[cfg(test)]
#[path = "serialize_test.rs"]
mod tests;
