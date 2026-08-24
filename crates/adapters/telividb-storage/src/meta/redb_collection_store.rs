//! The collection catalogue, persisted in `redb`.
//!
//! One file for the node rather than one per collection: the catalogue is
//! what answers "does this collection exist?", and a per-collection file
//! could only answer that by looking for itself — which is how a typo becomes
//! a new collection instead of an error.

use super::collection_record::{decode, encode};
use crate::error::Result;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use telividb_core::{Collection, ResourceName};
use telividb_telemetry::{fields, logger};

/// `resource name -> encoded catalogue entry`.
///
/// Versioned in the table name for the same reason a segment header carries a
/// version (rule 4).
const COLLECTIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("collections_v1");

/// The catalogue, backed by one `redb` file.
pub struct RedbCollectionStore {
    db: Database,
    /// Registration in the shared residency registry, released on drop.
    _resident: telividb_telemetry::residency::Handle,
}

impl RedbCollectionStore {
    /// Open the catalogue at `path`, creating it and any missing parent.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path).map_err(redb::Error::from)?;
        let write = db.begin_write().map_err(redb::Error::from)?;
        {
            write.open_table(COLLECTIONS).map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;

        let bytes = std::fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
        let _resident = telividb_telemetry::residency::register(
            telividb_telemetry::residency::ResidentKind::PointStore,
            telividb_telemetry::residency::Location::Host,
            path.display().to_string(),
            bytes,
        );
        logger::debug!("catalogue opened").with_data(&serde_json::json!({
            fields::STORE: "collection",
            fields::BACKEND: "redb",
            fields::RESIDENT_BYTES: bytes,
        }));

        Ok(Self { db, _resident })
    }

    /// Persist a new collection. Returns `false` without writing if one with
    /// this name already exists — this is `Create`, not `Upsert`.
    pub fn create(&self, collection: &Collection, descriptor_set: &[u8]) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let created = {
            let mut table = write.open_table(COLLECTIONS).map_err(redb::Error::from)?;
            let key = collection.name.as_str();
            if table.get(key).map_err(redb::Error::from)?.is_some() {
                false
            } else {
                table
                    .insert(key, encode(collection, descriptor_set).as_slice())
                    .map_err(redb::Error::from)?;
                true
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(created)
    }

    /// Fetch one collection, without its descriptor set.
    pub fn get(&self, name: &ResourceName) -> Result<Option<Collection>> {
        Ok(self.entry(name)?.map(|(collection, _)| collection))
    }

    /// Fetch one collection together with the descriptor bytes it was created
    /// from — the authoritative schema, stored verbatim.
    pub fn entry(&self, name: &ResourceName) -> Result<Option<(Collection, Vec<u8>)>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(COLLECTIONS).map_err(redb::Error::from)?;
        match table.get(name.as_str()).map_err(redb::Error::from)? {
            Some(value) => Ok(Some(decode(name.clone(), value.value())?)),
            None => Ok(None),
        }
    }

    /// Whether a collection exists, without decoding it.
    ///
    /// Separate from [`Self::get`] because the point path calls it on every
    /// write, and decoding a descriptor set to answer a yes/no question would
    /// read hundreds of kilobytes per point.
    pub fn exists(&self, name: &ResourceName) -> Result<bool> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(COLLECTIONS).map_err(redb::Error::from)?;
        Ok(table.get(name.as_str()).map_err(redb::Error::from)?.is_some())
    }

    /// Every collection, name-ordered.
    pub fn list(&self) -> Result<Vec<Collection>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let table = read.open_table(COLLECTIONS).map_err(redb::Error::from)?;

        let mut out = Vec::new();
        for row in table.iter().map_err(redb::Error::from)? {
            let (key, value) = row.map_err(redb::Error::from)?;
            let name = ResourceName::parse(key.value()).map_err(|e| {
                crate::error::Error::Domain(telividb_core::Error::PointStore {
                    reason: e.to_string(),
                })
            })?;
            out.push(decode(name, value.value())?.0);
        }
        Ok(out)
    }

    /// Remove a collection from the catalogue. Returns whether it existed.
    ///
    /// Removes the entry only. The collection's data directory is deleted by
    /// the caller, which is the one that knows where it lives — and which can
    /// report a partial deletion rather than leaving the catalogue and the
    /// filesystem disagreeing silently.
    pub fn delete(&self, name: &ResourceName) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let existed = {
            let mut table = write.open_table(COLLECTIONS).map_err(redb::Error::from)?;
            table
                .remove(name.as_str())
                .map_err(redb::Error::from)?
                .is_some()
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(existed)
    }
}
