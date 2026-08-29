//! Points, persisted in `redb`.
//!
//! One table, keyed by the point's resource name directly — no composite key
//! needed, unlike edges, since a point's name is already the natural key and
//! is unique by construction (AIP-122). The value format lives in
//! `point_record.rs`, split out to keep this file about the store's
//! read/write surface rather than the encoding.
//!
//! Table name carries a version suffix (`points_v1`) for the same reason the
//! edge table's does — CLAUDE.md rule 4, a redb table treated the way a
//! segment header already is.

use super::record::encode;
use crate::error::Result;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use telividb_core::{Point, ResourceName};
use telividb_telemetry::{fields, logger};

/// `resource name -> encoded point`. The read half lives in `point_read.rs`,
/// which is why this is visible to the module rather than to this file alone.
pub(super) const POINTS: TableDefinition<&str, &[u8]> = TableDefinition::new("points_v1");

/// `(field, row) -> point name`, so a search hit resolves back to a resource.
///
/// Necessary because a hit is a *segment-local ordinal* (invariant 9) which
/// means nothing outside the segment that produced it. `VectorField::row_of`
/// turns that into a field-wide row; this turns the row into the only
/// identity that may cross a process boundary — the point's resource name.
///
/// Keyed on `(field, row)` rather than row alone because each named vector
/// field numbers its rows independently.
pub(super) const ROWS: TableDefinition<(&str, u64), &str> = TableDefinition::new("vector_rows_v1");

/// Point storage backed by one `redb` database file.
///
/// Read-only access goes through [`PointStore`]; `create` and `delete` are
/// plain methods here, not part of that trait — the same split `GraphStore`
/// draws between reading a store and the separate writes that populate one.
pub struct RedbPointStore {
    pub(super) db: Database,
    /// Registration in the shared residency registry, released on drop.
    ///
    /// Sized by the backing file, so an operator listing what is resident sees
    /// this store beside the indexes and models competing for the same host.
    /// `Location::Host`: a redb file is page cache and system memory, and must
    /// not shrink the device ceiling a GPU index draws on.
    _resident: telividb_telemetry::residency::Handle,
}

impl RedbPointStore {
    /// Open the store at `path`, creating an empty one — and any missing
    /// parent directory, e.g. a collection's directory on its first point —
    /// if it does not exist.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path).map_err(redb::Error::from)?;
        let write = db.begin_write().map_err(redb::Error::from)?;
        {
            write.open_table(POINTS).map_err(redb::Error::from)?;
            write.open_table(ROWS).map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;

        // Size after creation, so a brand-new store registers its real (small)
        // footprint rather than zero.
        let bytes = std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        let _resident = telividb_telemetry::residency::register(
            telividb_telemetry::residency::ResidentKind::PointStore,
            telividb_telemetry::residency::Location::Host,
            path.display().to_string(),
            bytes,
        );
        logger::debug!("point opened").with_data(&serde_json::json!({
            fields::STORE: "point",
            fields::BACKEND: "redb",
            fields::RESIDENT_BYTES: bytes,
        }));

        Ok(Self { db, _resident })
    }

    /// Whether a point with this name exists, without decoding its record.
    pub fn exists(&self, name: &ResourceName) -> Result<bool> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(POINTS).map_err(redb::Error::from)?;
        Ok(table
            .get(name.as_str())
            .map_err(redb::Error::from)?
            .is_some())
    }

    /// Persist a new point. Returns `false`, without writing, if a point
    /// with this name already exists — this is `Create`, not `Upsert`.
    /// A bool rather than an error, mirroring `delete`: "already exists" is
    /// an ordinary outcome the caller decides how to report (AIP-133 wants
    /// `ALREADY_EXISTS`, not an opaque internal failure), not a storage
    /// failure.
    pub fn create(&self, point: &Point) -> Result<bool> {
        self.create_with_rows(point, &[])
    }

    /// Persist a point together with the vector rows that belong to it, in one
    /// transaction.
    ///
    /// Atomic on purpose: writing the bindings separately leaves a window where
    /// a crash produces rows pointing at a point that does not exist, and a
    /// search would then resolve a hit to a missing resource.
    pub fn create_with_rows(&self, point: &Point, rows: &[(String, u64)]) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let created = {
            let mut table = write.open_table(POINTS).map_err(redb::Error::from)?;
            let key = point.name.as_str();
            if table.get(key).map_err(redb::Error::from)?.is_some() {
                false
            } else {
                table
                    .insert(key, encode(point).as_slice())
                    .map_err(redb::Error::from)?;
                let mut bindings = write.open_table(ROWS).map_err(redb::Error::from)?;
                for (field, row) in rows {
                    bindings
                        .insert((field.as_str(), *row), key)
                        .map_err(redb::Error::from)?;
                }
                true
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(created)
    }

    /// Remove a point. Returns whether it existed.
    pub fn delete(&self, name: &ResourceName) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let existed = {
            let mut table = write.open_table(POINTS).map_err(redb::Error::from)?;
            let existed = table
                .remove(name.as_str())
                .map_err(redb::Error::from)?
                .is_some();

            // Clear this point's row bindings in the same transaction. Left
            // behind, they would resolve a later search hit to a resource that
            // no longer exists.
            let mut bindings = write.open_table(ROWS).map_err(redb::Error::from)?;
            let stale: Vec<(String, u64)> = bindings
                .iter()
                .map_err(redb::Error::from)?
                .filter_map(|row| row.ok())
                .filter(|(_, owner)| owner.value() == name.as_str())
                .map(|(key, _)| {
                    let (field, row) = key.value();
                    (field.to_owned(), row)
                })
                .collect();
            for (field, row) in stale {
                bindings
                    .remove((field.as_str(), row))
                    .map_err(redb::Error::from)?;
            }
            existed
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(existed)
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
