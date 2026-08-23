//! Tracking which nodes a search has already seen.
//!
//! The obvious implementation — `vec![false; n]` per search — allocates and
//! zeroes the whole corpus on **every layer visit**. During a build that is one
//! allocation per node per layer, so it grows as O(n²) in the corpus size and
//! dominates everything else: it turned a 50k-row build into a minute of
//! memset.
//!
//! Instead, allocate once and stamp each slot with an epoch. Clearing is an
//! increment.

/// An epoch-stamped visited set. Clearing is O(1).
#[derive(Debug)]
pub struct VisitedSet {
    stamps: Vec<u32>,
    epoch: u32,
}

impl VisitedSet {
    pub fn with_capacity(rows: usize) -> Self {
        Self {
            stamps: vec![0; rows],
            epoch: 0,
        }
    }

    /// Begin a new search. Every node becomes unvisited.
    pub fn clear(&mut self) {
        // On wraparound the stale stamps could alias the new epoch, so zero
        // once — every 4 billion searches, which is cheap enough to ignore.
        let (next, overflowed) = self.epoch.overflowing_add(1);
        if overflowed {
            self.stamps.fill(0);
            self.epoch = 1;
        } else {
            self.epoch = next;
        }
    }

    /// Mark `row` visited, returning `true` if it had not been seen this search.
    pub fn visit(&mut self, row: usize) -> bool {
        match self.stamps.get_mut(row) {
            Some(stamp) if *stamp != self.epoch => {
                *stamp = self.epoch;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "visited_test.rs"]
mod tests;
