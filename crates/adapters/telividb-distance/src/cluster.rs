//! Lloyd's algorithm, shared by everything that clusters vectors.
//!
//! **Why it lives here.** Two callers need it and neither may depend on the
//! other: product quantization in `telividb-storage` clusters *subspaces* into
//! codebook centroids, and the IVF coarse quantizer in `telividb-index`
//! clusters *whole vectors* into inverted lists. `telividb-distance` is the one
//! crate both already depend on, and clustering is distance-driven — this
//! module's inner loop is precisely the kernel the crate exists to provide.
//!
//! It also removes a duplicate: the k-means here used to carry its own squared
//! Euclidean function alongside the crate's [`l2_squared`].
//!
//! Deliberately small and deterministic rather than general. Seeding means the
//! same training set always yields the same centroids, and that reproducibility
//! matters more than it might seem — a codebook is baked into every vector
//! encoded against it, so a nondeterministic trainer means two builds of the
//! same data produce mutually unreadable codes.

use crate::ops::VectorOps;
use crate::rng::Rng;

/// Squared Euclidean distance, as training always measures it.
///
/// Always L2 regardless of the collection's metric: clustering approximates the
/// *vectors*, and reconstruction error is a Euclidean notion. Which metric
/// scores the result is a separate question, answered at query time.
///
/// Delegates to the crate's own kernel so there is one implementation to
/// optimise rather than two that must agree.
fn l2(a: &[f32], b: &[f32]) -> f32 {
    a.l2_squared(b)
}

/// Cluster `points` into `k` centroids.
///
/// Returns centroids laid out contiguously, `k * dim` floats.
pub(crate) fn train_impl(
    points: &[&[f32]],
    dim: usize,
    k: usize,
    iterations: usize,
    seed: u64,
) -> Vec<f32> {
    let mut rng = Rng(seed);
    let mut centroids = seed_centroids(points, dim, k, &mut rng);
    let mut assignment = vec![0usize; points.len()];

    for _ in 0..iterations {
        let mut moved = false;
        for (i, point) in points.iter().enumerate() {
            let nearest = nearest_centroid(point, &centroids, dim);
            if assignment[i] != nearest {
                assignment[i] = nearest;
                moved = true;
            }
        }
        // Converged: another pass would change nothing.
        if !moved {
            break;
        }
        update(&mut centroids, points, &assignment, dim, k, &mut rng);
    }
    centroids
}

/// k-means++ seeding: pick spread-out starting points.
///
/// Random seeding regularly puts two centroids in the same dense region and
/// leaves a whole cluster unrepresented, which shows up later as a subspace
/// that reconstructs badly for a fraction of the corpus.
fn seed_centroids(points: &[&[f32]], dim: usize, k: usize, rng: &mut Rng) -> Vec<f32> {
    let mut centroids = vec![0f32; k * dim];
    if points.is_empty() {
        return centroids;
    }

    let first = rng.below(points.len());
    centroids[..dim].copy_from_slice(points[first]);

    let mut best = vec![f32::INFINITY; points.len()];
    for c in 1..k {
        let previous = &centroids[(c - 1) * dim..c * dim];
        let mut total = 0f32;
        for (i, point) in points.iter().enumerate() {
            best[i] = best[i].min(l2(point, previous));
            total += best[i];
        }

        // Choose proportionally to squared distance from what is already chosen.
        let target = if total > 0.0 {
            (rng.next_u64() as f64 / u64::MAX as f64) as f32 * total
        } else {
            0.0
        };
        let mut running = 0f32;
        let mut chosen = points.len() - 1;
        for (i, &d) in best.iter().enumerate() {
            running += d;
            if running >= target {
                chosen = i;
                break;
            }
        }
        centroids[c * dim..(c + 1) * dim].copy_from_slice(points[chosen]);
    }
    centroids
}

/// Index of the centroid nearest `point`, by squared L2.
///
/// Squared rather than actual distance: the square root is monotonic, so it
/// cannot change which centroid wins, and this runs once per point per
/// iteration of training.
///
/// Returns zero when `centroids` is empty, which callers must not rely on:
/// every caller rejects an empty training set before reaching here, because a
/// degenerate codebook assigns every vector the same code and then ranks
/// nothing — silently.
pub fn nearest_centroid(point: &[f32], centroids: &[f32], dim: usize) -> usize {
    let mut best = 0usize;
    let mut best_distance = f32::INFINITY;
    for (c, centroid) in centroids.chunks(dim).enumerate() {
        let d = l2(point, centroid);
        if d < best_distance {
            best_distance = d;
            best = c;
        }
    }
    best
}

/// Move each centroid to the mean of its members.
fn update(
    centroids: &mut [f32],
    points: &[&[f32]],
    assignment: &[usize],
    dim: usize,
    k: usize,
    rng: &mut Rng,
) {
    let mut sums = vec![0f32; k * dim];
    let mut counts = vec![0usize; k];

    for (point, &cluster) in points.iter().zip(assignment) {
        counts[cluster] += 1;
        for (slot, &v) in sums[cluster * dim..(cluster + 1) * dim]
            .iter_mut()
            .zip(*point)
        {
            *slot += v;
        }
    }

    for c in 0..k {
        if counts[c] == 0 {
            // An empty cluster wastes a code point. Reseed it onto a random
            // point so the codebook keeps its full resolution rather than
            // silently shrinking to fewer usable centroids.
            if !points.is_empty() {
                let pick = rng.below(points.len());
                centroids[c * dim..(c + 1) * dim].copy_from_slice(points[pick]);
            }
            continue;
        }
        let n = counts[c] as f32;
        for (slot, &sum) in centroids[c * dim..(c + 1) * dim]
            .iter_mut()
            .zip(&sums[c * dim..(c + 1) * dim])
        {
            *slot = sum / n;
        }
    }
}

#[cfg(test)]
#[path = "cluster_test.rs"]
mod tests;
