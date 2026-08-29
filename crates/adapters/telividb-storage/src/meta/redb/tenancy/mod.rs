//! The tenancy tree: organizations, projects, spaces and sessions.
//!
//! Records here are Cap'n Proto rather than a hand-written byte layout — the
//! same schema the gRPC surface uses, so a stored organization and a wire
//! organization cannot become two definitions that must agree.
//!
//! One table per resource kind, keyed by resource name. Nothing here removes a
//! row: a delete stamps a tombstone and reads skip it, which is what makes
//! `undelete` possible at all.

pub mod organization;
pub mod project;
pub mod session;
pub mod space;
pub mod store;

mod children;
mod mutate;
mod time;
