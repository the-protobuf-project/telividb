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
    /// A set covering `rows`, with nothing visited.
    pub fn with_capacity(rows: usize) -> Self {
        Self {
            stamps: vec![0; rows],
            epoch: 0,
        }
    }

    /// Grow to cover `rows` if it does not already.
    ///
    /// Never shrinks: a pooled set is reused across fields and segments of
    /// different sizes, and returning capacity only to reallocate it on the
    /// next query is the cost this pool exists to avoid. New slots are zero,
    /// which reads as "not visited this epoch" for any epoch.
    pub fn resize(&mut self, rows: usize) {
        if self.stamps.len() < rows {
            self.stamps.resize(rows, 0);
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

/// Visited sets kept for reuse across searches.
///
/// `search` takes `&self` and runs concurrently, so this cannot be a plain
/// field. It is a small mutex-guarded free list: a query takes a set, uses it,
/// and gives it back. Under contention a query simply allocates its own rather
/// than waiting, which keeps the lock off the critical path.
///
/// The alternative — `VisitedSet::with_capacity(store.len())` per query —
/// allocates and zeroes four bytes per row on every search. `visited.rs`
/// documents that exact pattern as the trap that made builds quadratic; it is
/// no cheaper on the read path, it is merely charged per query instead of per
/// insert.
#[derive(Debug, Default)]
pub struct ScratchPool {
    /// Sets not currently in use.
    free: std::sync::Mutex<Vec<VisitedSet>>,
}

/// How many sets to keep. Beyond this, a returned set is dropped.
///
/// Bounded because the pool would otherwise grow to the high-water mark of
/// concurrent queries and hold that memory forever — on a wide field that is
/// megabytes per retained set.
const POOL_CAPACITY: usize = 16;

impl ScratchPool {
    /// A set sized for `rows`, cleared and ready.
    pub fn take(&self, rows: usize) -> VisitedSet {
        let reused = self.free.lock().ok().and_then(|mut free| free.pop());
        match reused {
            Some(mut set) => {
                set.resize(rows);
                set.clear();
                set
            }
            None => VisitedSet::with_capacity(rows),
        }
    }

    /// Return a set for the next query to use.
    pub fn give_back(&self, set: VisitedSet) {
        if let Ok(mut free) = self.free.lock()
            && free.len() < POOL_CAPACITY
        {
            free.push(set);
        }
    }
}

#[cfg(test)]
#[path = "visited_test.rs"]
mod tests;
