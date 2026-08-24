use super::*;
use telividb_core::{ContentRef, Span};

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
