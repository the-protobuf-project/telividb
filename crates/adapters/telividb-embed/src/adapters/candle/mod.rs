//! The candle adapter — the only implementation of [`Inferencer`].
//!
//! [`Inferencer`]: crate::ports::Inferencer

mod batch;
mod bert;
mod config;
mod device;
#[cfg(test)]
mod fixture;
mod inferencer;
mod model;
mod pipeline;
mod schedule;
mod tokenize;
mod weights;

pub use batch::Batch;
pub use device::{best_device, device_name};
pub use inferencer::CandleInferencer;
pub use model::ResidentModel;
