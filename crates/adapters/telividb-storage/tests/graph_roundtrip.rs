//! Persist edges, reopen the file, rehydrate, traverse.
//!
//! This is the "done when" test for Plan A1.1's first slice: `RedbGraphStore`
//! and `telividb_graph::Graph` never depend on each other in production code
//! (`GraphStore` lives in `telividb-core` precisely so they don't have to),
//! so this is the one place that proves the two halves actually fit. The
//! reopen goes through `open_graph_store`, not `RedbGraphStore` directly, so
//! this also proves rehydration works against the generic `dyn GraphStore`
//! the factory returns — swapping the backend later touches only the
//! `GraphStoreConfig` value passed in, never this test.

use telividb_core::{Edge, ResourceName};
use telividb_graph::Graph;
use telividb_storage::{GraphStoreConfig, RedbGraphStore, open_graph_store};

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn edges_survive_a_close_and_reopen_then_traverse_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("meta.redb");

    // Write with one handle, then drop it — the graph must be readable from a
    // fresh open, not just within the writer's own lifetime.
    {
        let store = RedbGraphStore::open(&path).unwrap();
        store
            .insert_edge(&Edge::new(name("video"), name("shot-1"), "HAS_SHOT", 1.0))
            .unwrap();
        store
            .insert_edge(&Edge::new(name("video"), name("shot-2"), "HAS_SHOT", 1.0))
            .unwrap();
        store
            .insert_edge(&Edge::new(
                name("shot-1"),
                name("entity-a"),
                "MENTIONS",
                1.0,
            ))
            .unwrap();
    }

    let config = GraphStoreConfig::Redb { path };
    let reopened = open_graph_store(&config).unwrap();
    let graph = Graph::rehydrate(reopened.as_ref()).unwrap();

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 3);

    let one_hop = {
        let mut reached = graph.k_hop(&name("video"), 1, None, None);
        reached.sort();
        reached
    };
    assert_eq!(one_hop, vec![name("shot-1"), name("shot-2")]);

    let two_hop = {
        let mut reached = graph.k_hop(&name("video"), 2, None, None);
        reached.sort();
        reached
    };
    assert_eq!(
        two_hop,
        vec![name("entity-a"), name("shot-1"), name("shot-2")]
    );

    // Filtering by edge type stops the walk at shot-2, which has no
    // outbound HAS_SHOT edges of its own.
    let has_shot_only = graph.k_hop(&name("video"), 2, Some("HAS_SHOT"), None);
    assert_eq!(has_shot_only.len(), 2, "HAS_SHOT never reaches entity-a");
}
