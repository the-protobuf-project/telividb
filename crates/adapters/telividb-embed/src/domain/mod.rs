//! Vocabulary of the inference boundary. No I/O, no tensors.

mod model_id;
mod pooling;
mod task;

pub use model_id::ModelId;
pub use pooling::Pooling;
pub use task::Task;
