//! How far a seed expands, and what that costs its score.

/// The shape of one expansion.
#[derive(Debug, Clone)]
pub struct Expansion {
    /// Edges to traverse from each seed. Zero returns the seeds alone, which
    /// is an ordinary configuration rather than a degenerate one — it is what
    /// "search without graph context" means.
    pub hops: usize,
    /// Restrict traversal to one edge type; `None` follows every edge.
    ///
    /// The difference between "what else is in this document" and "what does
    /// this mention", and getting it wrong floods the result with structurally
    /// related but semantically irrelevant neighbours.
    pub edge_type: Option<String>,
    /// Score multiplier applied once per hop.
    ///
    /// **Without this, expansion drowns the search.** A seed with a hundred
    /// neighbours contributes a hundred results at its own score, so the graph
    /// half swamps the similarity half and ranking stops meaning anything.
    /// Decay encodes the thing that is actually true: a neighbour is evidence,
    /// and weaker evidence than a direct match.
    pub decay: f32,
    /// Ceiling on results returned from traversal, across all seeds.
    ///
    /// A hub node — an entity mentioned by every document — expands without
    /// bound. This is what stops one of them from becoming the entire answer.
    pub max_expanded: usize,
    /// Whether a larger score is a better one, as the seeding metric defines it.
    ///
    /// Carried rather than assumed because the join has to merge and rank, and
    /// L2 ranks *lower* as nearer while dot and cosine rank higher. A planner
    /// that guessed would silently invert the results for one metric family.
    pub higher_is_nearer: bool,
}

impl Default for Expansion {
    fn default() -> Self {
        Self {
            hops: 1,
            edge_type: None,
            // A neighbour at roughly two-thirds the weight of a direct hit:
            // enough to surface as context, not enough to outrank the thing
            // that was actually searched for.
            decay: 0.65,
            max_expanded: 64,
            higher_is_nearer: true,
        }
    }
}

impl Expansion {
    /// Traverse `hops` edges from each seed.
    pub fn hops(mut self, hops: usize) -> Self {
        self.hops = hops;
        self
    }

    /// Follow only edges of this type.
    pub fn along(mut self, edge_type: impl Into<String>) -> Self {
        self.edge_type = Some(edge_type.into());
        self
    }

    /// Weight each hop by `decay`.
    pub fn decaying(mut self, decay: f32) -> Self {
        self.decay = decay;
        self
    }

    /// Return at most `max` expanded results.
    pub fn limited_to(mut self, max: usize) -> Self {
        self.max_expanded = max;
        self
    }

    /// Rank under a metric where lower scores are nearer, such as L2.
    pub fn lower_is_nearer(mut self) -> Self {
        self.higher_is_nearer = false;
        self
    }

    /// Order two scores best-first under this expansion's metric direction.
    pub fn rank(&self, a: f32, b: f32) -> std::cmp::Ordering {
        match self.higher_is_nearer {
            true => b.total_cmp(&a),
            false => a.total_cmp(&b),
        }
    }

    /// A seed's score after `hops` edges.
    ///
    /// Multiplicative where higher is nearer; **divisive where lower is**, so
    /// decay always weakens. Multiplying an L2 distance by 0.65 would make a
    /// two-hop neighbour look *nearer* than the seed that found it, which is
    /// exactly backwards.
    pub fn decayed(&self, score: f32, hops: usize) -> f32 {
        let factor = self.decay.powi(hops as i32);
        match self.higher_is_nearer {
            true => score * factor,
            false => match factor > 0.0 {
                true => score / factor,
                false => f32::MAX,
            },
        }
    }
}

#[cfg(test)]
#[path = "expansion_test.rs"]
mod tests;
