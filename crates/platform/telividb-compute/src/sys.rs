//! The generated ggml declarations.
//!
//! Produced by `bindgen` from ggml's own headers at build time, so they cannot
//! drift from the C API. Nothing outside this crate may name anything in here —
//! every item is a raw pointer or an untagged union, and the safe types beside
//! it exist precisely so callers never hold one.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![allow(dead_code, missing_docs, clippy::all)]

include!(concat!(env!("OUT_DIR"), "/ggml.rs"));
