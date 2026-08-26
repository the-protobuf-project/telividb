//! Loading a real GGUF onto a device.
//!
//! Against the actual model rather than a synthetic fixture, because the things
//! most likely to be wrong here are things a fixture would not reproduce: the
//! quantized block types, the tensor naming convention llama.cpp's converter
//! emits, and whether metadata keys are where the architecture says they are.
//!
//! Skipped rather than failed when the model is absent — it is 80 MiB and not
//! committed (`examples/models/download.sh`). A skipped test says so out loud
//! so it cannot be mistaken for a passing one.

use crate::{Backend, DeviceKind, Weights};
use std::path::PathBuf;

/// The downloaded text model, if it is present.
fn model() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/models/gguf/text/nomic-embed-text-v1.5.Q4_K_M.gguf");
    path.exists().then_some(path)
}

#[test]
fn a_real_gguf_loads_onto_the_device() {
    let Some(path) = model() else {
        eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
        return;
    };
    let backend = Backend::of(DeviceKind::Cpu).unwrap();
    let weights = Weights::load(&path, &backend).unwrap();

    assert!(
        !weights.is_empty(),
        "a model with no tensors is not a model"
    );
    // Every embedding model has a vocabulary table; its absence means the
    // naming convention is not what this loader assumes.
    assert!(
        weights.raw_tensor("token_embd.weight").is_some(),
        "no token_embd.weight among {} tensors",
        weights.len()
    );
}

#[test]
fn architecture_parameters_are_read_from_the_file() {
    let Some(path) = model() else {
        eprintln!("SKIPPED: run examples/models/download.sh to exercise this");
        return;
    };
    let backend = Backend::of(DeviceKind::Cpu).unwrap();
    let weights = Weights::load(&path, &backend).unwrap();

    // Read, never assumed: a wrong layer count produces finite, correctly
    // shaped, wrong vectors — the failure mode with no symptom.
    let blocks = weights
        .u32_meta("nomic-bert.block_count")
        .expect("block_count is absent — check the architecture prefix");
    assert!(
        (1..=64).contains(&blocks),
        "implausible block count {blocks}"
    );

    // Every declared block must actually be present, which is what catches a
    // truncated download that still parses.
    for i in 0..blocks {
        assert!(
            weights
                .raw_tensor(&format!("blk.{i}.attn_output.weight"))
                .is_some(),
            "block {i} of {blocks} is missing its attention output"
        );
    }
}

#[test]
fn a_missing_file_is_an_error_rather_than_a_panic() {
    let backend = Backend::of(DeviceKind::Cpu).unwrap();
    let err = Weights::load(std::path::Path::new("/nonexistent/model.gguf"), &backend);
    assert!(err.is_err());
}
