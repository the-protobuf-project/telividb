//! Tests for the tenancy store.
//!
//! The delete-then-undelete round trip is the one that matters: every resource
//! in this tree carries `delete_time` and `expire_time` precisely so a delete
//! can be taken back, and nothing else in the codebase exercises that path.

use super::*;
use telividb_core::Lifecycle;

/// A store in a fresh temporary directory, and the path to clean up.
fn store(tag: &str) -> (RedbTenancyStore, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("telividb-tenancy-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("tenancy.redb");
    (RedbTenancyStore::open(&path).expect("opens"), dir)
}

/// A live organization named `organizations/{id}`.
fn org(id: &str) -> Organization {
    Organization {
        name: ResourceName::parse(format!("organizations/{id}")).expect("a valid name"),
        display_name: id.to_owned(),
        lifecycle: Lifecycle {
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
            deleted_at: None,
            expires_at: None,
        },
    }
}

#[test]
fn create_is_not_upsert() {
    let (store, dir) = store("create");
    let acme = org("acme");

    assert!(store.create_organization(&acme).expect("first create"));
    // A second create must not overwrite: the name is taken, and silently
    // replacing would hand one tenant's identity to another's data.
    assert!(!store.create_organization(&acme).expect("second create"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn delete_then_undelete_restores_the_organization() {
    let (store, dir) = store("undelete");
    let acme = org("acme");
    store.create_organization(&acme).expect("create");

    let deleted = store
        .delete_organization(&acme.name, 1_700_000_500_000)
        .expect("delete")
        .expect("something was deleted");
    assert!(!deleted.lifecycle.is_live());
    assert_eq!(deleted.lifecycle.deleted_at, Some(1_700_000_500_000));
    // Recoverable until it expires, which is what makes the next step possible.
    assert!(deleted.lifecycle.expires_at.is_some());

    let restored = store
        .undelete_organization(&acme.name, 1_700_000_600_000)
        .expect("undelete")
        .expect("something was restored");
    assert!(restored.lifecycle.is_live());
    assert_eq!(restored.lifecycle.expires_at, None);
    assert_eq!(restored.display_name, acme.display_name);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tombstone_is_hidden_from_an_ordinary_list() {
    let (store, dir) = store("list");
    store.create_organization(&org("live")).expect("create");
    store.create_organization(&org("gone")).expect("create");
    let gone = ResourceName::parse("organizations/gone").expect("a valid name");
    store
        .delete_organization(&gone, 1_700_000_500_000)
        .expect("delete");

    let visible = store.organizations(false).expect("list");
    assert_eq!(visible.len(), 1, "a tombstone reached an ordinary list");

    // The undelete screen needs to find it, so the tombstone is still there.
    let all = store.organizations(true).expect("list all");
    assert_eq!(all.len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn deleting_twice_reports_nothing_the_second_time() {
    let (store, dir) = store("twice");
    let acme = org("acme");
    store.create_organization(&acme).expect("create");
    store
        .delete_organization(&acme.name, 1)
        .expect("first delete");

    // Already tombstoned. Restamping would move the expiry, quietly extending
    // the life of something a caller believes it already deleted.
    let again = store
        .delete_organization(&acme.name, 2)
        .expect("second delete");
    assert!(again.is_none());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn undeleting_something_that_was_never_deleted_reports_nothing() {
    let (store, dir) = store("noop");
    let acme = org("acme");
    store.create_organization(&acme).expect("create");
    assert!(
        store
            .undelete_organization(&acme.name, 2)
            .expect("undelete")
            .is_none()
    );
    let _ = std::fs::remove_dir_all(&dir);
}
