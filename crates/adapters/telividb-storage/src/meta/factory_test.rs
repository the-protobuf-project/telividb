use super::*;
use telividb_core::{Edge, Point, ResourceName};

#[test]
fn redb_config_opens_a_working_store() {
    let dir = tempfile::tempdir().unwrap();
    let config = GraphStoreConfig::Redb {
        path: dir.path().join("meta.redb"),
    };

    // The returned value is `Box<dyn GraphStore>` — this compiles at all only
    // because the factory never leaks `RedbGraphStore` into this scope.
    let store: Box<dyn GraphStore> = open_graph_store(&config).unwrap();
    assert!(store.iter_edges().unwrap().next().is_none());
}

#[test]
fn a_store_opened_through_the_factory_sees_edges_written_before_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("meta.redb");

    // `insert_edge` is the concrete adapter's own write method, not part of
    // `GraphStore` — the port stays read-only, matching `VectorStore` — so
    // the fixture is written with a direct handle, then dropped. Two live
    // `redb::Database` handles on the same file at once is not a scenario
    // this test needs to exercise.
    {
        let writer = RedbGraphStore::open(&path).unwrap();
        writer
            .insert_edge(&Edge::new(
                ResourceName::parse("a").unwrap(),
                ResourceName::parse("b").unwrap(),
                "NEXT",
                1.0,
            ))
            .unwrap();
    }

    let config = GraphStoreConfig::Redb { path };
    let store = open_graph_store(&config).unwrap();
    let edges: Vec<_> = store.iter_edges().unwrap().collect();
    assert_eq!(edges.len(), 1);
}

#[test]
fn point_store_config_opens_a_working_store() {
    let dir = tempfile::tempdir().unwrap();
    let config = PointStoreConfig::Redb {
        path: dir.path().join("points.redb"),
    };

    // Same proof as the graph config above, for the point side: this
    // compiles only because the factory returns `Box<dyn PointStore>`.
    let store: Box<dyn PointStore> = open_point_store(&config).unwrap();
    assert!(
        store
            .get(&ResourceName::parse("collections/a/points/1").unwrap())
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_point_created_directly_is_visible_through_the_factory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("points.redb");
    let name = ResourceName::parse("collections/a/points/1").unwrap();

    {
        let writer = RedbPointStore::open(&path).unwrap();
        writer.create(&Point::new(name.clone())).unwrap();
    }

    let config = PointStoreConfig::Redb { path };
    let store = open_point_store(&config).unwrap();
    assert!(store.get(&name).unwrap().is_some());
}
