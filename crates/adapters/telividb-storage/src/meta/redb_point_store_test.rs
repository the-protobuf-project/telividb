use super::*;
use telividb_core::{ContentRef, PointStore, Span};

fn name(s: &str) -> ResourceName {
    ResourceName::parse(s).unwrap()
}

#[test]
fn open_creates_a_missing_parent_directory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("media").join("points.redb");
    // `media/` does not exist yet — this is the shape `PointsSvc` actually
    // opens: one subdirectory per collection, none of them pre-created.
    RedbPointStore::open(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn getting_a_missing_point_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    assert!(
        store
            .get(&name("collections/media/points/x"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_created_point_round_trips_through_get() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    let point = Point::new(name("collections/media/points/doc-1"))
        .with_span(Span::new(0, 100).unwrap())
        .with_content_ref(ContentRef::uri("s3://bucket/key"));
    store.create(&point).unwrap();

    let fetched = store.get(&point.name).unwrap().unwrap();
    assert_eq!(fetched, point);
}

#[test]
fn creating_the_same_name_twice_reports_false_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    let point = Point::new(name("a/points/1"));
    assert!(store.create(&point).unwrap(), "first create succeeds");
    assert!(
        !store.create(&point).unwrap(),
        "second create reports false, not an error"
    );
}

#[test]
fn list_returns_only_direct_children_of_parent() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    store
        .create(&Point::new(name("collections/a/points/1")))
        .unwrap();
    store
        .create(&Point::new(name("collections/a/points/2")))
        .unwrap();
    store
        .create(&Point::new(name("collections/b/points/1")))
        .unwrap();

    let mut listed: Vec<_> = store
        .list(&name("collections/a"))
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    listed.sort();
    assert_eq!(
        listed,
        vec![
            name("collections/a/points/1"),
            name("collections/a/points/2")
        ]
    );
}

#[test]
fn delete_reports_whether_the_point_existed() {
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    let point = Point::new(name("a/points/1"));
    store.create(&point).unwrap();

    assert!(store.delete(&point.name).unwrap());
    assert!(!store.delete(&point.name).unwrap(), "already gone");
    assert!(store.get(&point.name).unwrap().is_none());
}

#[test]
fn a_bound_row_resolves_back_to_its_point() {
    // The mapping that keeps invariant 9 honest: a search returns a
    // segment-local ordinal, and only this turns it into a portable identity.
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    let point = name("collections/media/points/doc-1");

    store.bind_row("text_bge", 7, &point).unwrap();
    assert_eq!(store.row_owner("text_bge", 7).unwrap(), Some(point));
}

#[test]
fn an_unbound_row_has_no_owner() {
    // Ordinary, not exceptional: a crash between appending a vector and
    // binding its row looks exactly like this.
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    assert!(store.row_owner("text_bge", 0).unwrap().is_none());
}

#[test]
fn rows_are_numbered_per_field() {
    // Each named vector field numbers its rows independently, so the same row
    // number in two fields is two different points.
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    let a = name("collections/media/points/a");
    let b = name("collections/media/points/b");

    store.bind_row("text_bge", 0, &a).unwrap();
    store.bind_row("image_clip", 0, &b).unwrap();

    assert_eq!(store.row_owner("text_bge", 0).unwrap(), Some(a));
    assert_eq!(store.row_owner("image_clip", 0).unwrap(), Some(b));
}

#[test]
fn a_binding_survives_a_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("points.redb");
    let point = name("collections/media/points/doc-1");
    {
        let store = RedbPointStore::open(&path).unwrap();
        store.bind_row("text_bge", 3, &point).unwrap();
    }
    let reopened = RedbPointStore::open(&path).unwrap();
    assert_eq!(reopened.row_owner("text_bge", 3).unwrap(), Some(point));
}

#[test]
fn list_excludes_nested_names_that_merely_share_the_prefix() {
    // `collections/a/points/1/parts/2` starts with the same prefix but is a
    // different resource one level deeper; AIP-132 lists direct children.
    let dir = tempfile::tempdir().unwrap();
    let store = RedbPointStore::open(&dir.path().join("points.redb")).unwrap();
    store
        .create(&Point::new(name("collections/a/points/1")))
        .unwrap();
    store
        .create(&Point::new(name("collections/a/points/1/parts/2")))
        .unwrap();

    let listed: Vec<_> = store
        .list(&name("collections/a"))
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(listed, vec![name("collections/a/points/1")]);
}
