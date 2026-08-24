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

use super::point_record::{decode, encode};
use crate::error::Result;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use telividb_core::{Point, PointStore, ResourceName};
use telividb_telemetry::{fields, logger};

const POINTS: TableDefinition<&str, &[u8]> = TableDefinition::new("points_v1");

/// `(field, row) -> point name`, so a search hit resolves back to a resource.
///
/// Necessary because a hit is a *segment-local ordinal* (invariant 9) which
/// means nothing outside the segment that produced it. `VectorField::row_of`
/// turns that into a field-wide row; this turns the row into the only
/// identity that may cross a process boundary — the point's resource name.
///
/// Keyed on `(field, row)` rather than row alone because each named vector
/// field numbers its rows independently.
const ROWS: TableDefinition<(&str, u64), &str> = TableDefinition::new("vector_rows_v1");

/// Point storage backed by one `redb` database file.
///
/// Read-only access goes through [`PointStore`]; `create` and `delete` are
/// plain methods here, not part of that trait — the same split `GraphStore`
/// draws between reading a store and the separate writes that populate one.
pub struct RedbPointStore {
    db: Database,
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

    /// Persist a new point. Returns `false`, without writing, if a point
    /// with this name already exists — this is `Create`, not `Upsert`.
    /// A bool rather than an error, mirroring `delete`: "already exists" is
    /// an ordinary outcome the caller decides how to report (AIP-133 wants
    /// `ALREADY_EXISTS`, not an opaque internal failure), not a storage
    /// failure.
    pub fn create(&self, point: &Point) -> Result<bool> {
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
                true
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(created)
    }

    /// Record that `field`'s row `row` belongs to the point named `name`.
    pub fn bind_row(&self, field: &str, row: u64, name: &ResourceName) -> Result<()> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        {
            let mut table = write.open_table(ROWS).map_err(redb::Error::from)?;
            table
                .insert((field, row), name.as_str())
                .map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;
        Ok(())
    }

    /// The point a field's row belongs to, if that row was ever bound.
    ///
    /// `None` is ordinary rather than exceptional: a row can exist in a
    /// segment while its binding is absent, which is what a crash between the
    /// vector append and this write looks like.
    pub fn row_owner(&self, field: &str, row: u64) -> Result<Option<ResourceName>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(ROWS).map_err(redb::Error::from)?;
        let found = table.get((field, row)).map_err(redb::Error::from)?;
        Ok(found.and_then(|v| ResourceName::parse(v.value()).ok()))
    }

    /// Remove a point. Returns whether it existed.
    pub fn delete(&self, name: &ResourceName) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let existed = {
            let mut table = write.open_table(POINTS).map_err(redb::Error::from)?;
            table
                .remove(name.as_str())
                .map_err(redb::Error::from)?
                .is_some()
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(existed)
    }
}

impl PointStore for RedbPointStore {
    fn get(&self, name: &ResourceName) -> telividb_core::Result<Option<Point>> {
        let read = self.db.begin_read().map_err(point_store_err)?;
        let table = read.open_table(POINTS).map_err(point_store_err)?;
        match table.get(name.as_str()).map_err(point_store_err)? {
            Some(value) => Ok(Some(decode(name.clone(), value.value())?)),
            None => Ok(None),
        }
    }

    fn list(&self, parent: &ResourceName) -> telividb_core::Result<Vec<Point>> {
        let prefix = format!("{}/points/", parent.as_str());
        let read = self.db.begin_read().map_err(point_store_err)?;
        let table = read.open_table(POINTS).map_err(point_store_err)?;

        let mut points = Vec::new();
        let range = table
            .range::<&str>(prefix.as_str()..)
            .map_err(point_store_err)?;
        for row in range {
            let (key, value) = row.map_err(point_store_err)?;
            if !key.value().starts_with(&prefix) {
                break;
            }
            let name =
                ResourceName::parse(key.value()).map_err(|e| telividb_core::Error::PointStore {
                    reason: e.to_string(),
                })?;
            points.push(decode(name, value.value())?);
        }
        Ok(points)
    }
}

fn point_store_err<E: Into<redb::Error>>(e: E) -> telividb_core::Error {
    telividb_core::Error::PointStore {
        reason: e.into().to_string(),
    }
}

#[cfg(test)]
#[path = "redb_point_store_test.rs"]
mod tests;
