//! The tenancy tree, persisted in `redb`.
//!
//! One file for the node, one table per resource kind, keyed by resource name
//! and valued as a Cap'n Proto record — the same schema the gRPC surface uses,
//! so a stored organization and a wire organization cannot become two
//! definitions that must agree.
//!
//! # Delete is soft, everywhere
//!
//! Nothing here removes a row. A delete stamps `deleted_at` and an expiry, and
//! reads skip it unless asked otherwise. That is what makes `undelete` possible
//! at all, and it is why the protos carry `delete_time` and `expire_time` on
//! every resource in this tree rather than treating deletion as an absence.

use super::record::{decode, encode};
use crate::error::Result;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use telividb_core::{Organization, ResourceName};
use telividb_telemetry::{fields, logger};

/// `resource name -> encoded organization`.
///
/// Versioned in the table name for the same reason a segment header carries a
/// version (rule 4): a future layout is refused rather than misread.
pub(super) const ORGANIZATIONS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("organizations_v1");

/// How long a soft-deleted resource stays recoverable: thirty days in millis.
///
/// Long enough that a mistaken delete is noticed by a person on holiday, short
/// enough that the bytes are not kept forever.
pub(super) const RETENTION_MILLIS: i64 = 30 * 24 * 60 * 60 * 1_000;

/// The tenancy tree, backed by one `redb` file.
pub struct RedbTenancyStore {
    /// The database. One file, several tables.
    pub(super) db: Database,
    /// Registration in the shared residency registry, released on drop.
    _resident: telividb_telemetry::residency::Handle,
}

impl RedbTenancyStore {
    /// Open the tree at `path`, creating it and any missing parent.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path).map_err(redb::Error::from)?;
        let write = db.begin_write().map_err(redb::Error::from)?;
        {
            write.open_table(ORGANIZATIONS).map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;

        let bytes = std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        let _resident = telividb_telemetry::residency::register(
            telividb_telemetry::residency::ResidentKind::PointStore,
            telividb_telemetry::residency::Location::Host,
            path.display().to_string(),
            bytes,
        );
        logger::debug!("tenancy store opened").with_data(&serde_json::json!({
            fields::STORE: "tenancy",
            fields::BACKEND: "redb",
            fields::RESIDENT_BYTES: bytes,
        }));

        Ok(Self { db, _resident })
    }

    /// Persist a new organization.
    ///
    /// Returns `false` without writing if one with this name already exists —
    /// this is `Create`, not `Upsert`. A soft-deleted organization still holds
    /// its name, so creating over one is refused too: the name is taken until
    /// it expires, and quietly reusing it would resurrect a tenant's identity
    /// under someone else's data.
    pub fn create_organization(&self, org: &Organization) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let created = {
            let mut table = write.open_table(ORGANIZATIONS).map_err(redb::Error::from)?;
            let key = org.name.as_str();
            if table.get(key).map_err(redb::Error::from)?.is_some() {
                false
            } else {
                table
                    .insert(key, encode(org).as_slice())
                    .map_err(redb::Error::from)?;
                true
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(created)
    }

    /// Fetch one organization, including a soft-deleted one.
    ///
    /// The caller decides whether a tombstone is what it wanted — `undelete`
    /// needs to find one, and a read path needs to skip it.
    pub fn organization(&self, name: &ResourceName) -> Result<Option<Organization>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(ORGANIZATIONS).map_err(redb::Error::from)?;
        match table.get(name.as_str()).map_err(redb::Error::from)? {
            Some(value) => Ok(Some(decode(name.clone(), value.value())?)),
            None => Ok(None),
        }
    }

    /// Every organization, in resource-name order.
    ///
    /// Ordered because `redb` iterates its keys sorted and the key is the
    /// resource name — so the order is stable across calls without sorting
    /// afterwards, which a paginated list will need.
    ///
    /// `show_deleted` includes tombstones, which is what an undelete screen
    /// needs and what an ordinary list must not show.
    pub fn organizations(&self, show_deleted: bool) -> Result<Vec<Organization>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(ORGANIZATIONS).map_err(redb::Error::from)?;

        let mut out = Vec::new();
        for row in table.iter().map_err(redb::Error::from)? {
            let (key, value) = row.map_err(redb::Error::from)?;
            let name = ResourceName::parse(key.value()).map_err(|e| {
                crate::error::Error::Domain(telividb_core::Error::PointStore {
                    reason: e.to_string(),
                })
            })?;
            let org = decode(name, value.value())?;
            if show_deleted || org.lifecycle.is_live() {
                out.push(org);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
