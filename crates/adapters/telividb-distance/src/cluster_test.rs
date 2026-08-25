use super::*;
use crate::l2_squared;

/// Three well-separated blobs in 2D.
fn blobs() -> Vec<Vec<f32>> {
    let mut points = Vec::new();
    for (cx, cy) in [(0.0f32, 0.0f32), (10.0, 10.0), (-10.0, 8.0)] {
        for i in 0..20 {
            let jitter = i as f32 * 0.01;
            points.push(vec![cx + jitter, cy - jitter]);
        }
    }
    points
}

fn as_refs(points: &[Vec<f32>]) -> Vec<&[f32]> {
    points.iter().map(Vec::as_slice).collect()
}

#[test]
fn recovers_well_separated_clusters() {
    let points = blobs();
    let centroids = train(&as_refs(&points), 2, 3, 25, 7);

    // Every point should sit near the centroid it was assigned to.
    for point in &points {
        let c = nearest_centroid(point, &centroids, 2);
        let distance = l2_squared(point, &centroids[c * 2..(c + 1) * 2]);
        assert!(
            distance < 1.0,
            "point {point:?} is {distance} from its centroid"
        );
    }
}

#[test]
fn training_is_deterministic() {
    // A codebook is baked into every vector encoded against it, so two builds
    // of the same data must produce identical, mutually readable codes.
    let points = blobs();
    let a = train(&as_refs(&points), 2, 3, 25, 7);
    let b = train(&as_refs(&points), 2, 3, 25, 7);
    assert_eq!(a, b);
}

#[test]
fn a_different_seed_may_differ_but_still_converges() {
    let points = blobs();
    let centroids = train(&as_refs(&points), 2, 3, 25, 99);
    for point in &points {
        let c = nearest_centroid(point, &centroids, 2);
        assert!(l2_squared(point, &centroids[c * 2..(c + 1) * 2]) < 1.0);
    }
}

#[test]
fn seeding_spreads_centroids_across_clusters() {
    // Random seeding regularly drops two centroids in one dense region and
    // leaves a cluster unrepresented, which later reads as a subspace that
    // reconstructs badly for part of the corpus.
    let points = blobs();
    let centroids = train(&as_refs(&points), 2, 3, 25, 7);

    let mut used = std::collections::HashSet::new();
    for point in &points {
        used.insert(nearest_centroid(point, &centroids, 2));
    }
    assert_eq!(used.len(), 3, "a centroid went unused");
}

#[test]
fn more_centroids_than_points_does_not_panic() {
    let points = vec![vec![1.0f32, 2.0], vec![3.0, 4.0]];
    let centroids = train(&as_refs(&points), 2, 8, 10, 1);
    assert_eq!(centroids.len(), 16);
}

#[test]
fn an_empty_training_set_yields_zeroed_centroids() {
    let centroids = train(&[], 4, 3, 10, 1);
    assert_eq!(centroids.len(), 12);
    assert!(centroids.iter().all(|&x| x == 0.0));
}

#[test]
fn identical_points_do_not_hang() {
    // Every distance is zero, so the proportional draw has nothing to weight.
    let points = vec![vec![5.0f32; 4]; 30];
    let centroids = train(&as_refs(&points), 4, 4, 20, 3);
    assert!(centroids.iter().all(|x| x.is_finite()));
}

#[test]
fn nearest_centroid_picks_the_closest() {
    let centroids = vec![0.0, 0.0, 10.0, 10.0];
    assert_eq!(nearest_centroid(&[0.1, 0.1], &centroids, 2), 0);
    assert_eq!(nearest_centroid(&[9.5, 9.5], &centroids, 2), 1);
}
