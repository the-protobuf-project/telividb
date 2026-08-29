//! Projects, spaces and sessions.
//!
//! Split from the organization's own tests because these cover the tree
//! *beneath* the root: that four kinds share one soft-delete path without
//! sharing a table, and that the two fields with consequences — a space's
//! protection and a session's absent space — survive a round trip.

// Included from `children.rs`, so `super` is that module and its imports come
// with the glob.
use super::*;
use telividb_core::{Lifecycle, Organization, Protection};

/// A store in a fresh temporary directory, and the path to clean up.
fn store(tag: &str) -> (RedbTenancyStore, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("telividb-children-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("tenancy.redb");
    (RedbTenancyStore::open(&path).expect("opens"), dir)
}

/// A live organization.
fn org(id: &str) -> Organization {
    Organization {
        name: ResourceName::parse(format!("organizations/{id}")).expect("a valid name"),
        display_name: id.to_owned(),
        lifecycle: Lifecycle::default(),
    }
}

/// A live project under `organizations/acme`.
fn project(id: &str) -> Project {
    Project {
        name: ResourceName::parse(format!("organizations/acme/projects/{id}"))
            .expect("a valid name"),
        display_name: id.to_owned(),
        lifecycle: Lifecycle {
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
            deleted_at: None,
            expires_at: None,
        },
    }
}

/// A live space with a declared protection state.
fn space(id: &str, protection: Protection) -> Space {
    Space {
        name: ResourceName::parse(format!("organizations/acme/spaces/{id}")).expect("a valid name"),
        display_name: id.to_owned(),
        projects: vec![
            ResourceName::parse("organizations/acme/projects/atlas").expect("a valid name"),
        ],
        protection,
        lifecycle: Lifecycle {
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
            deleted_at: None,
            expires_at: None,
        },
    }
}

#[test]
fn every_resource_in_the_tree_round_trips() {
    let (store, dir) = store("tree");

    store.create_organization(&org("acme")).expect("org");
    store.create_project(&project("atlas")).expect("project");
    store
        .create_space(&space("finance", Protection::Private))
        .expect("space");
    store
        .create_session(&Session {
            name: ResourceName::parse("organizations/acme/sessions/s-1").expect("a valid name"),
            display_name: "Tuesday".to_owned(),
            space: Some(
                ResourceName::parse("organizations/acme/spaces/finance").expect("a valid name"),
            ),
            lifecycle: Lifecycle::default(),
        })
        .expect("session");

    assert_eq!(store.organizations(false).expect("orgs").len(), 1);
    assert_eq!(store.projects(false).expect("projects").len(), 1);
    assert_eq!(store.spaces(false).expect("spaces").len(), 1);
    assert_eq!(store.sessions(false).expect("sessions").len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_space_keeps_its_protection_and_its_projects() {
    let (store, dir) = store("protection");
    let vault = space("board", Protection::Vault);
    store.create_space(&vault).expect("create");

    // Protection decides segment routing, so a record that lost it would put a
    // vault's points in with everyone else's — and nothing downstream would
    // notice, because the points would be perfectly readable.
    let read = store
        .space(&vault.name)
        .expect("read")
        .expect("the space exists");
    assert_eq!(read.protection, Protection::Vault);
    assert_eq!(read.projects, vault.projects);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_session_without_a_space_round_trips_as_absent() {
    let (store, dir) = store("orphan");
    let loose = Session {
        name: ResourceName::parse("organizations/acme/sessions/s-2").expect("a valid name"),
        display_name: String::new(),
        // Cap'n Proto text has no null, so absence is the empty string — and
        // "" is not a resource name, so the two cannot be confused.
        space: None,
        lifecycle: Lifecycle::default(),
    };
    store.create_session(&loose).expect("create");

    let read = store
        .session(&loose.name)
        .expect("read")
        .expect("the session exists");
    assert_eq!(read.space, None);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn delete_and_undelete_work_for_every_kind() {
    let (store, dir) = store("softdelete");
    store.create_organization(&org("acme")).expect("org");
    store.create_project(&project("atlas")).expect("project");
    store
        .create_space(&space("finance", Protection::None))
        .expect("space");

    let p = project("atlas").name;
    let s = space("finance", Protection::None).name;

    assert!(store.delete_project(&p, 100).expect("delete").is_some());
    assert!(store.delete_space(&s, 100).expect("delete").is_some());
    assert_eq!(store.projects(false).expect("live").len(), 0);
    assert_eq!(store.spaces(false).expect("live").len(), 0);

    // Recoverable until expiry, which is the whole reason nothing is removed.
    assert!(store.undelete_project(&p, 200).expect("undelete").is_some());
    assert!(store.undelete_space(&s, 200).expect("undelete").is_some());
    assert_eq!(store.projects(false).expect("live").len(), 1);
    assert_eq!(store.spaces(false).expect("live").len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}
