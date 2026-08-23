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
    /// An empty graph with no nodes and no entry point.
    pub fn new() -> Self {
        Self::default()
    }

    /// Rows the graph covers, including absent ones.
    ///
    /// This is the store's row count, not the number of searchable nodes: an
    /// absent row still occupies a slot so the ordinal space stays dense.
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// Whether the graph covers no rows at all.
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// The row every descent starts from.
    ///
    /// `None` only for an empty graph. It is always a row with a vector —
    /// see [`Graph::push_absent`] for why that matters.
    pub fn entry(&self) -> Option<Ordinal> {
        self.entry.map(Ordinal::from_row)
    }

    /// The top layer, which is the layer the entry point sits on.
    pub fn max_level(&self) -> usize {
        self.max_level
    }

    /// The highest layer `node` appears on; zero for an unknown node.
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
    ///
    /// Only for rows that actually have a vector for this field. A row without
    /// one must go through [`Graph::push_absent`]: it cannot be scored, so as
    /// an entry point it strands every insert that follows.
    pub fn push_node(&mut self, level: usize) -> u32 {
        let node = self.push_slot(level);
        if self.entry.is_none() || level > self.max_level {
            self.entry = Some(node);
            self.max_level = level;
        }
        node
    }

    /// Add a placeholder for a row with no vector for this field.
    ///
    /// The row still occupies an ordinal so the fixed stride holds, but it is
    /// not a node in the graph and — critically — **never becomes the entry
    /// point**. An absent entry cannot be scored, so `distance_to` returns
    /// `None` and every subsequent insert bails out before linking anything.
    /// With an absent row at the head of a field that stranded present rows
    /// until some later node happened to draw a higher level and take over.
    pub fn push_absent(&mut self) -> u32 {
        self.push_slot(0)
    }

    /// Reserve the storage one row occupies, without touching the entry point.
    fn push_slot(&mut self, level: usize) -> u32 {
        let node = self.levels.len() as u32;
        self.levels.push(level);
        self.links.push(vec![Vec::new(); level + 1]);
        node
    }

    /// Set the entry point and top layer directly.
    ///
    /// For [`super::serialize::decode`], which restores what the header
    /// recorded rather than inferring it by replaying inserts. Inference cannot
    /// work on a decoded graph: an absent row and a present row that drew level
    /// zero are indistinguishable once written, so replay would pick the wrong
    /// entry for exactly the graphs [`Graph::push_absent`] exists to protect.
    pub fn set_entry(&mut self, entry: Option<u32>, max_level: usize) {
        self.entry = entry;
        self.max_level = max_level;
    }

    /// Replace `node`'s neighbours at `layer`, ignoring an out-of-range node.
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
