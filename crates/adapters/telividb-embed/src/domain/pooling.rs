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
    /// Take the last real token's state, ignoring padding.
    ///
    /// What decoder-derived embedders are trained for — Qwen3-Embedding among
    /// them. Attention is causal there, so only the final position has seen the
    /// whole sequence; the first token has seen nothing but itself, which is
    /// why reading one of these with [`Cls`](Self::Cls) is not a near miss but
    /// a vector built from a single word.
    ///
    /// "Last real" rather than "last": the state at a padding position is not
    /// part of the text, and taking it would make a vector depend on how much
    /// padding its batch happened to need.
    Last,
}

impl Pooling {
    /// What the model itself declares, when it declares anything.
    ///
    /// GGUF records this as `<arch>.pooling_type`, using llama.cpp's
    /// numbering: 1 is mean, 2 is CLS. Reading it beats asking the caller,
    /// because a caller who guesses wrong gets vectors of the right width in
    /// the right range that simply rank badly — there is no error to notice.
    ///
    /// `None` when the key is absent or names something that is not a pooled
    /// embedding: `0` is "no pooling" and `4` is a reranker's score head. The
    /// caller then has to decide, which is the honest outcome — better than
    /// defaulting quietly to whichever mode is more common.
    pub fn from_declared(value: u32) -> Option<Self> {
        match value {
            1 => Some(Pooling::Mean),
            2 => Some(Pooling::Cls),
            3 => Some(Pooling::Last),
            _ => None,
        }
    }

    /// The name used in configuration and telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            Pooling::Mean => "mean",
            Pooling::Cls => "cls",
            Pooling::Last => "last",
        }
    }
}
