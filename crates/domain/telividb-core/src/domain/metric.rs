//! Distance metrics.

/// How similarity is measured within a named vector field.
///
/// `Cosine` is normalised at ingest and evaluated as [`Metric::Dot`] thereafter,
/// so nothing normalises per query on the hot path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// Inner product. Higher is nearer.
    Dot,
    /// Squared Euclidean distance. Lower is nearer.
    L2,
    /// Angular similarity. Stored normalised, scored as `Dot`.
    Cosine,
}

impl Metric {
    /// Whether a larger score means a nearer neighbour.
    pub fn higher_is_nearer(self) -> bool {
        match self {
            Metric::Dot | Metric::Cosine => true,
            Metric::L2 => false,
        }
    }

    /// Whether vectors must be unit-normalised when written.
    pub fn normalises_at_ingest(self) -> bool {
        matches!(self, Metric::Cosine)
    }
}
