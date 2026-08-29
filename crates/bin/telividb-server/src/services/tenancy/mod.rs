//! The tenancy services: organizations, projects, spaces and sessions.
//!
//! One store behind four services. They share a `redb` file because they are
//! one tree — a project without its organization is not a smaller thing, it is
//! a broken one — and sharing the file is what lets a delete and the read that
//! follows it agree.

mod convert;
mod organizations;
mod projects;
mod sessions;
mod spaces;

use std::path::Path;
use std::sync::Arc;
use telividb_storage::RedbTenancyStore;
use tonic::Status;

/// The tenancy tree, shared by every service in this module.
#[derive(Clone)]
pub struct TenancySvc {
    /// The store. `Arc` because four services hold it and `tonic` needs each
    /// to be `Send + Sync + 'static`.
    store: Arc<RedbTenancyStore>,
}

impl TenancySvc {
    /// Open the tree under `data_dir`.
    ///
    /// Once, at construction rather than per request: `redb` takes an
    /// exclusive file lock, so two concurrent opens of the same file fail.
    pub fn open(data_dir: &Path) -> Result<Self, telividb_storage::Error> {
        let store = RedbTenancyStore::open(&data_dir.join("tenancy.redb"))?;
        Ok(Self {
            store: Arc::new(store),
        })
    }
}

/// Now, in milliseconds since the Unix epoch.
///
/// Taken from the system clock at the moment of the write rather than from the
/// request. A client-supplied time would let one machine's clock skew decide
/// when another's data expires.
fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// Parse a resource name, or refuse with the reason.
fn parse(raw: &str) -> std::result::Result<telividb_core::ResourceName, Status> {
    telividb_core::ResourceName::parse(raw).map_err(|e| Status::invalid_argument(e.to_string()))
}

/// Refuse an empty id with a message naming what it is for.
fn require_id(id: &str, field: &str) -> std::result::Result<(), Status> {
    match id.is_empty() {
        true => Err(Status::invalid_argument(format!(
            "{field} must not be empty: it forms the final path segment of the \
             resource's name, and a name is permanent"
        ))),
        false => Ok(()),
    }
}

/// The status for a resource that is already there.
fn already_exists(name: &telividb_core::ResourceName) -> Status {
    Status::already_exists(format!(
        "{name} already exists. A soft-deleted resource still holds its name \
         until it expires — undelete it rather than creating over it."
    ))
}

/// The status for a resource that is not.
fn not_found(name: &telividb_core::ResourceName) -> Status {
    Status::not_found(format!("{name} not found"))
}
