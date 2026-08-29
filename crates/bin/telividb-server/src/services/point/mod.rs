//! The points service: writing, reading and searching a collection's rows.

mod batch;
mod convert;
mod create;
mod declare;
mod delete;
mod search;
mod service;
mod store;

pub use service::PointsSvc;
