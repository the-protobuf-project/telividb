use super::*;
use telividb_core::Edge;

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn open_creates_a_missing_parent_directory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("media").join("meta.redb");
    RedbGraphStore::open(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn a_fresh_store_has_no_edges() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbGraphStore::open(&dir.path().join("meta.redb")).unwrap();
    let edges: Vec<_> = store.iter_edges().unwrap().collect();
    assert!(edges.is_empty());
}

#[test]
fn an_inserted_edge_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbGraphStore::open(&dir.path().join("meta.redb")).unwrap();
    let edge = Edge::new(name("a/1"), name("b/1"), "MENTIONS", 0.75);
    store.insert_edge(&edge).unwrap();

    let edges: Vec<Edge> = store
        .iter_edges()
        .unwrap()
        .collect::<telividb_core::Result<_>>()
        .unwrap();
    assert_eq!(edges, vec![edge]);
}

#[test]
fn two_edge_types_between_the_same_pair_both_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbGraphStore::open(&dir.path().join("meta.redb")).unwrap();
    store
        .insert_edge(&Edge::new(name("a"), name("b"), "MENTIONS", 1.0))
        .unwrap();
    store
        .insert_edge(&Edge::new(name("a"), name("b"), "CO_OCCURS", 1.0))
        .unwrap();

    let mut edges: Vec<Edge> = store
        .iter_edges()
        .unwrap()
        .collect::<telividb_core::Result<_>>()
        .unwrap();
    edges.sort_by(|a, b| a.edge_type.cmp(&b.edge_type));
    assert_eq!(edges[0].edge_type, "CO_OCCURS");
    assert_eq!(edges[1].edge_type, "MENTIONS");
}

#[test]
fn a_malformed_key_is_reported_on_that_row_not_the_whole_scan() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbGraphStore::open(&dir.path().join("meta.redb")).unwrap();
    store
        .insert_edge(&Edge::new(name("a"), name("b"), "OK", 1.0))
        .unwrap();

    // Write a row with too few NUL-separated fields directly, bypassing
    // `insert_edge`'s always-well-formed encoding.
    let write = store.db.begin_write().unwrap();
    {
        let mut table = write.open_table(EDGES).unwrap();
        table
            .insert("no-separator-here", [0u8; 4].as_slice())
            .unwrap();
    }
    write.commit().unwrap();

    let results: Vec<_> = store.iter_edges().unwrap().collect();
    assert_eq!(results.len(), 2);
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
}

#[test]
fn reopening_the_same_file_sees_prior_edges() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("meta.redb");

    {
        let store = RedbGraphStore::open(&path).unwrap();
        store
            .insert_edge(&Edge::new(name("a"), name("b"), "NEXT", 1.0))
            .unwrap();
    }

    let reopened = RedbGraphStore::open(&path).unwrap();
    let edges: Vec<_> = reopened.iter_edges().unwrap().collect();
    assert_eq!(edges.len(), 1);
}
