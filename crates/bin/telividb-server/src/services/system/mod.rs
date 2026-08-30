//! What this process found when it started, served rather than inferred.
//!
//! A build that quietly fell back to the CPU passes every correctness test while
//! delivering none of the speed — from outside the pod is healthy, the GPU is
//! allocated, and nothing is using it. The selected backend is the one fact no
//! orchestrator can see from outside, so it is served here.
//!
//! **This is the server's answer, not the window's.** The desktop app detected
//! the same facts in its own process, which worked only because it links the
//! engine; a browser talking to a Linux daemon had no way to ask at all. Serving
//! it is what makes the two deployments agree (see *Two deployments, one engine*
//! in CLAUDE.md).

mod info;

pub use info::SystemSvc;
