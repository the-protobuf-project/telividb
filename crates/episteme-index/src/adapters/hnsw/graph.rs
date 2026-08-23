//! The layered proximity graph.

use episteme_core::Ordinal;

/// Adjacency for every node, at every layer it appears on.
///
/// Stored as `Vec<Vec<Vec<u32>>>` — node, then layer, then neighbours — which
/// is the shape the build mutates. The serialized form is flat and
/// offset-addressed so it can be read straight from a mapped file; converting
/// between the two is the job of the serializer, not this type.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// `links[node][layer]` — neighbours of `node` at `layer`.
    links: Vec<Vec<Vec<u32>>>,
    /// Highest layer each node appears on.
    levels: Vec<usize>,
    /// Node the descent starts from: the one with the highest level.
    entry: Option<u32>,
    max_level: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn entry(&self) -> Option<Ordinal> {
        self.entry.map(Ordinal::from_row)
    }

    pub fn max_level(&self) -> usize {
        self.max_level
    }

    pub fn level_of(&self, node: u32) -> usize {
        self.levels.get(node as usize).copied().unwrap_or(0)
    }

    /// Neighbours of `node` at `layer`; empty if the node does not reach it.
    pub fn neighbours(&self, node: u32, layer: usize) -> &[u32] {
        self.links
            .get(node as usize)
            .and_then(|layers| layers.get(layer))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Add a node occupying layers `0..=level`.
    ///
    /// Becomes the entry point when it reaches higher than anything so far —
    /// the descent must start from the top or the upper layers are unreachable.
    pub fn push_node(&mut self, level: usize) -> u32 {
        let node = self.levels.len() as u32;
        self.levels.push(level);
        self.links.push(vec![Vec::new(); level + 1]);

        if self.entry.is_none() || level > self.max_level {
            self.entry = Some(node);
            self.max_level = level;
        }
        node
    }

    pub fn set_neighbours(&mut self, node: u32, layer: usize, neighbours: Vec<u32>) {
        if let Some(layers) = self.links.get_mut(node as usize)
            && let Some(slot) = layers.get_mut(layer)
        {
            *slot = neighbours;
        }
    }

    /// Add `neighbour` to `node`'s list at `layer` without exceeding `budget`.
    ///
    /// Returns whether the list is now full, which is the signal for the caller
    /// to re-run the selection heuristic and prune rather than simply dropping
    /// the newest edge — dropping would make connectivity depend on insertion
    /// order.
    pub fn try_connect(&mut self, node: u32, layer: usize, neighbour: u32, budget: usize) -> bool {
        let Some(layers) = self.links.get_mut(node as usize) else {
            return false;
        };
        let Some(list) = layers.get_mut(layer) else {
            return false;
        };
        if list.contains(&neighbour) {
            return list.len() >= budget;
        }
        list.push(neighbour);
        list.len() >= budget
    }

    /// Total edges, for diagnostics and for sizing the serialized form.
    pub fn edge_count(&self) -> usize {
        self.links
            .iter()
            .flat_map(|layers| layers.iter())
            .map(Vec::len)
            .sum()
    }
}

#[cfg(test)]
#[path = "graph_test.rs"]
mod tests;
