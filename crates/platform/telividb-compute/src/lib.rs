//! The tensor runtime — layer one.
//!
//! ggml, wrapped so that no other crate in the workspace contains `unsafe` or
//! sees a raw pointer. Every type here owns its allocation and frees it on
//! drop; every operation is a method on a value rather than a loose function
//! taking a handle.
//!
//! # Why this crate exists at all
//!
//! The layers above — distance kernels, clustering, the index families — are
//! pure Rust over `&[f32]` and know nothing about hardware. They stay that way
//! only if exactly one crate binds to a runtime. This is that crate.
//!
//! # Why ggml rather than a pure-Rust runtime
//!
//! Hardware coverage, and it is measurable rather than aesthetic. candle's
//! `Device` has three variants — CPU, CUDA, Metal — so Intel and AMD GPUs are
//! unreachable from it at any effort. ggml carries CUDA, Metal, Vulkan, HIP,
//! SYCL, OpenCL and more behind one backend interface.
//!
//! The cost is recorded rather than hidden: ggml is C/C++ built with CMake, so
//! this crate needs a native toolchain and contains `unsafe`. That is the whole
//! reason it is a crate of its own.
//!
//! # What belongs here, and what does not
//!
//! Only operations where a device genuinely wins: dense scoring of many rows
//! against a query, which is a matmul with high arithmetic intensity and no
//! data dependence. Selection, partitioning and graph traversal stay on the
//! CPU because they are branchy and dependent — measured at 25x slower on a
//! device for the scattered-gather case. **The device scores; the host
//! decides.**
#![deny(missing_docs)]

mod backend;
mod corpus;
mod device;
mod error;
mod graph;
mod score;
mod sys;

pub use backend::{Backend, Memory};
pub use corpus::Corpus;
pub use device::{Device, DeviceKind};
pub use error::{Error, Result};
pub use score::Scores;
