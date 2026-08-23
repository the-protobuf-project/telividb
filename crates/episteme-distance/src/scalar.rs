//! Reference implementations. Always correct; never the fast path.

/// Inner product. Higher means nearer, so callers negate it where a
/// distance is wanted.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Squared Euclidean distance. The square root is omitted deliberately — it is
/// monotonic, so it changes no ordering and costs a transcendental per compare.
pub fn l2_squared(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum()
}

/// Scale `v` to unit length in place. A zero vector is left untouched rather
/// than producing NaNs that would poison every later comparison.
pub fn normalize(v: &mut [f32]) {
    let norm = dot(v, v).sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
#[path = "scalar_test.rs"]
mod tests;
