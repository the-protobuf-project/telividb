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
//! # A failed `GGML_ASSERT` kills the process, and cannot be caught
//!
//! `GGML_ASSERT` expands to `abort()`. It does not return an error, does not
//! unwind, and no Rust code above it gets to run — so a shape this crate passes
//! down wrongly is not a bad `Result`, it is a dead server with an open socket.
//! **Every argument that ggml would assert on is therefore validated in Rust
//! before the call**, which is why the wrappers here look more defensive than
//! their C signatures suggest they need to be. That is deliberate; do not
//! "simplify" a check away because the C function appears to handle it.
//!
//! Upstream made the death less destructive on macOS rather than survivable:
//! ggml used to attach `lldb` to print a backtrace, which could take down
//! Terminal.app and every other window with it (llama.cpp#17869, in v0.22.0 —
//! the version this crate pins). The default is now a native `backtrace()`, and
//! the `lldb` path is opt-in through `GGML_BACKTRACE_LLDB`. **Nothing in this
//! workspace sets that variable and nothing should**, least of all CI: a crash
//! report is not worth a developer losing their session, and the native
//! backtrace names the frame either way.
//!

#![deny(missing_docs)]

mod arrays;
mod attention;
mod backend;
mod context;
mod corpus;
mod device;
mod error;
mod graph;
mod header;
mod input;
mod metadata;
mod score;
mod shape;
mod staged;
mod sys;
mod tensor;
mod upload;
mod weights;

pub use backend::{Backend, Memory};
pub use context::Context;
pub use corpus::Corpus;
pub use device::{Device, DeviceKind};
pub use error::{Error, Result};
pub use header::Header;
pub use score::Scores;
pub use staged::Staged;
pub use tensor::Tensor;
pub use weights::Weights;
