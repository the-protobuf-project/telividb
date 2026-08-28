//! Grouping texts into batches that do not waste work on padding.
//!
//! **The problem this solves.** A batch is padded to its longest member, and
//! attention is quadratic in that padded length. Batch a 12-token sentence
//! with a 2000-token abstract and the sentence is computed at 2000 tokens —
//! so a corpus of mixed lengths spends most of its time on padding it did not
//! need. Measured on BEIR SciFact, roughly six sevenths of the work was
//! padding: 1048 seconds where the real tokens accounted for about 170.
//!
//! Sorting by length first means each batch holds texts of similar size, so
//! the padding within it is small. The results are unchanged — padding is
//! masked out of both attention and pooling — so this is purely a matter of
//! not doing work that is then discarded.

/// One batch: indices into the caller's original input.
///
/// Indices rather than the texts themselves so the caller can scatter results
/// back into input order, which the [`Inferencer`] contract requires.
///
/// [`Inferencer`]: crate::ports::Inferencer
pub type Batch = Vec<usize>;

/// Plan batches over `lengths`, in tokens.
///
/// `token_budget` caps `rows * padded_length` per batch, which is what
/// actually drives memory and time — a fixed row count cannot, since 64 rows
/// of 8 tokens and 64 of 2000 differ by 250x in work.
///
/// `max_rows` bounds the row count as well, because at very short lengths the
/// budget alone would allow a batch of thousands, and per-row overhead starts
/// to dominate before the budget is reached.
pub fn plan(lengths: &[usize], token_budget: usize, max_rows: usize) -> Vec<Batch> {
    if lengths.is_empty() {
        return Vec::new();
    }

    // Longest first. A too-large budget then fails on the very first batch
    // rather than part-way through a long run, and neighbouring lengths end up
    // together either way.
    let mut order: Vec<usize> = (0..lengths.len()).collect();
    order.sort_by(|a, b| lengths[*b].cmp(&lengths[*a]).then(a.cmp(b)));

    let mut batches: Vec<Batch> = Vec::new();
    let mut current: Batch = Vec::new();
    // The first element of a descending run is its longest, so it is what the
    // rest of the batch pads up to.
    let mut padded = 0usize;

    for index in order {
        let candidate = padded.max(lengths[index]).max(1);
        let rows = current.len() + 1;

        // Never emit an empty batch: a single text longer than the whole
        // budget still has to run, and refusing it would make one long
        // document unembeddable.
        if !current.is_empty() && (candidate * rows > token_budget || rows > max_rows) {
            batches.push(std::mem::take(&mut current));
            padded = lengths[index].max(1);
        } else {
            padded = candidate;
        }
        current.push(index);
    }

    if !current.is_empty() {
        batches.push(current);
    }
    batches
}

#[cfg(test)]
#[path = "schedule_test.rs"]
mod tests;
