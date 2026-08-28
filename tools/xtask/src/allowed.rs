//! Which workspace crates each crate may name.
//!
//! Split from the checker because the two change for different reasons: this
//! is the architecture, and `check_layers.rs` is the mechanism that reads a
//! manifest and compares against it. A new crate edits this file and nothing
//! else.
//!
//! **An entry here is a design decision, not configuration.** Adding a name to
//! a list is the moment a layering rule is relaxed, and it should be as visible
//! in review as the dependency it permits.

/// What each crate is allowed to depend on, inside the workspace.
///
/// Listed rather than derived from a rank, because the interesting constraint
/// is not "lower number" but *which* boundary a crate may see: two crates at
/// the same depth are not interchangeable, and an adapter reaching sideways
/// into another adapter is precisely the edge rule 14 forbids.
pub(crate) const ALLOWED: &[(&str, &[&str])] = &[
    ("telividb-core", &[]),
    // Layer one. Depends only on the ontology — it must not know what an index
    // or a model is, or the layers above could not be swapped independently.
    ("telividb-compute", &["telividb-core"]),
    // Every rendering of the schema: protobuf and its gRPC stubs, plus the
    // Cap'n Proto and FlatBuffers views. Depends on nothing in the workspace and
    // must not — it is the wire format and nothing else, so a plugin author
    // outside this repo can take it without taking the engine.
    ("telividb-buffers", &[]),
    ("telividb-telemetry", &[]),
    ("telividb-graph", &["telividb-core", "telividb-telemetry"]),
    // The query planner composes *results*, not indexes — a caller runs its own
    // search and hands the hits in. That is what keeps it in `domain/`: it can
    // name the graph and the ontology and nothing outward, and it is testable
    // with no index, no storage and no device.
    (
        "telividb-query",
        &["telividb-core", "telividb-graph", "telividb-telemetry"],
    ),
    // Layer two may reach layer one for dense scoring, and nothing else.
    ("telividb-distance", &["telividb-core", "telividb-compute"]),
    (
        "telividb-storage",
        &["telividb-core", "telividb-distance", "telividb-telemetry"],
    ),
    (
        "telividb-index",
        &[
            "telividb-compute",
            "telividb-core",
            "telividb-distance",
            "telividb-telemetry",
        ],
    ),
    // Layer four reaches layer one, for the same reason layer two does: the
    // encoder builds its forward pass out of `telividb-compute` graph
    // operations, and a model runtime that could not name the tensor runtime
    // would have to carry its own.
    //
    // Note what is still absent: `telividb-index`. The inference server and
    // the device index both sit on `telividb-compute`, but neither may reach
    // into the other — a shared device helper would put one adapter behind the
    // other's optional feature, which is the outward dependency rule 14
    // forbids.
    (
        "telividb-embed",
        &["telividb-compute", "telividb-core", "telividb-telemetry"],
    ),
    // The SDK speaks the wire protocol and nothing else: no storage, no index,
    // no model. That is what keeps one server behaviour rather than one per
    // client, and it is why this list is as short as it is.
    ("telividb-client", &["telividb-buffers"]),
    (
        "telividb-server",
        &[
            "telividb-core",
            "telividb-distance",
            "telividb-embed",
            "telividb-index",
            "telividb-buffers",
            "telividb-storage",
            "telividb-telemetry",
        ],
    ),
];
