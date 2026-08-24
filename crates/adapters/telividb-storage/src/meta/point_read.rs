//! The read half of the point store — the [`PointStore`] port itself.
//!
//! Split from `redb_point_store.rs` because the two halves answer to different
//! owners: this one implements a `telividb-core` port and so must speak that
//! crate's error type, while the write surface next door is inherent API
//! speaking storage errors. Keeping them in one file meant every method paid
//! attention to which error vocabulary it was in.

use super::point_record::decode;
use super::redb_point_store::{POINTS, RedbPointStore};
use redb::ReadableDatabase;
use telividb_core::{Point, PointStore, ResourceName};

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
            let Some(suffix) = key.value().strip_prefix(&prefix) else {
                break;
            };
            // Direct children only. A nested name like
            // `collections/a/points/1/parts/2` shares the prefix but is a
            // different resource, and AIP-132 lists one level.
            if suffix.contains('/') {
                continue;
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

/// Restate a `redb` failure as the port's error, since a caller holding only
/// `telividb-core` cannot name `redb::Error`.
fn point_store_err<E: Into<redb::Error>>(e: E) -> telividb_core::Error {
    telividb_core::Error::PointStore {
        reason: e.into().to_string(),
    }
}
