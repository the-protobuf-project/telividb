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

use super::cursor::Cursor;
use super::graph::Graph;
use telividb_core::{Error, Result};

/// File header magic identifying an encoded HNSW graph. A mismatch means the
/// bytes are not this format at all, and `decode` refuses them.
pub const GRAPH_MAGIC: [u8; 4] = *b"EPHN";

/// Format version of the encoded layout. `decode` refuses any version it does
/// not recognize rather than guessing at a layout that may have changed.
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
            reason: "not an telividb hnsw graph",
        });
    }
    if cursor.u16()? > GRAPH_VERSION {
        return Err(Error::MalformedIndex {
            reason: "hnsw graph written by a newer telividb",
        });
    }
    cursor.take(2)?; // reserved

    let nodes = cursor.u64()? as usize;
    let declared_entry = cursor.u32()?;
    let declared_max_level = cursor.u32()? as usize;
    let declared_edges = cursor.u64()? as usize;

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
    // `push_node` rather than `push_absent`, then the entry is set from the
    // header below. An absent row and a present row that drew level zero are
    // indistinguishable once written, so replay cannot infer the entry point —
    // it has to be restored from what the writer recorded.
    for (level, _) in &shape {
        graph.push_node(*level);
    }

    // The header is a claim about the graph that follows, so it is checked
    // against it rather than discarded. Reconstruction happens to reproduce the
    // same entry for a well-formed file, which is exactly why a disagreeing
    // header decoded silently into a different graph before.
    let entry = match declared_entry {
        u32::MAX => None,
        row if (row as usize) < nodes => Some(row),
        _ => {
            return Err(Error::MalformedIndex {
                reason: "hnsw entry point is not a node in this graph",
            });
        }
    };
    if entry.is_none() && declared_edges != 0 {
        return Err(Error::MalformedIndex {
            reason: "hnsw graph has edges but declares no entry point",
        });
    }
    if let Some(row) = entry
        && shape[row as usize].0 != declared_max_level
    {
        return Err(Error::MalformedIndex {
            reason: "hnsw entry point is not on the declared top layer",
        });
    }
    if shape.iter().map(|(level, _)| *level).max().unwrap_or(0) != declared_max_level && nodes != 0
    {
        return Err(Error::MalformedIndex {
            reason: "hnsw max level disagrees with the node levels",
        });
    }
    let counted_edges: usize = shape.iter().flat_map(|(_, lengths)| lengths.iter()).sum();
    if counted_edges != declared_edges {
        return Err(Error::MalformedIndex {
            reason: "hnsw edge count disagrees with the per-layer lengths",
        });
    }
    graph.set_entry(entry, declared_max_level);
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

#[cfg(test)]
#[path = "serialize_test.rs"]
mod tests;
