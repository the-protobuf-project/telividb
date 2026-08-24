//! Turning per-token states into one vector.

/// How a sequence of token states collapses to a single embedding.
///
/// Declared per model rather than assumed, because the choice is not
/// interchangeable: a model trained with mean pooling and read with CLS
/// pooling returns vectors of the right width, in the right range, that
/// rank badly. Nothing errors, so nothing surfaces it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pooling {
    /// Average over real tokens, ignoring padding. What the BERT-family
    /// embedding models (bge, e5, gte, nomic) are trained for.
    Mean,
    /// Take the first token's state. What models with a trained `[CLS]`
    /// objective expect.
    Cls,
}

impl Pooling {
    /// The name used in configuration and telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            Pooling::Mean => "mean",
            Pooling::Cls => "cls",
        }
    }
}
