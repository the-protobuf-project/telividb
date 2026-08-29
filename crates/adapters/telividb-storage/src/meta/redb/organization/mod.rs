//! The tenancy tree.
//!
//! Records here are Cap'n Proto rather than a hand-written byte layout — the
//! same schema the gRPC surface uses, so a stored organization and a wire
//! organization cannot become two definitions that must agree.

pub mod lifecycle;
pub mod record;
pub mod store;
