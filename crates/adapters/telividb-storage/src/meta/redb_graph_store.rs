//! Edges, persisted in `redb`.
//!
//! One table, keyed by `"{src}\0{edge_type}\0{dst}"`. Resource names and edge
//! types never contain a NUL byte, so the separator cannot collide with real
//! data. NUL-delimiting rather than a length-prefixed encoding makes "every
//! edge from `src`" and "every edge from `src` of one type" cheap prefix range
//! scans over the table's own sort order — exactly what rehydration and,
//! later, k-hop expansion from storage would want.
//!
//! Table name carries a version suffix (`edges_v1`) so a future schema change
//! is `edges_v2`, not a silent reinterpretation of old rows — CLAUDE.md rule 4
//! applied to a `redb` table the way it already applies to segment headers.

use crate::error::Result;
use redb::{Database, ReadableDatabase, TableDefinition};
use std::path::Path;
use telividb_core::{Edge, GraphStore, ResourceName};
use telividb_telemetry::{fields, logger};

const EDGES: TableDefinition<&str, &[u8]> = TableDefinition::new("edges_v1");

/// Edge storage backed by one `redb` database file.
///
/// Read-only access goes through [`GraphStore`]; writing is a plain method
/// here, not part of that trait — the same split `VectorStore` draws between
/// reading a store and the separate writer that populates one.
pub struct RedbGraphStore {
    db: Database,
    /// Registration in the shared residency registry, released on drop.
    ///
    /// Sized by the backing file, so an operator listing what is resident sees
    /// this store beside the indexes and models competing for the same host.
    /// `Location::Host`: a redb file is page cache and system memory, and must
    /// not shrink the device ceiling a GPU index draws on.
    _resident: telividb_telemetry::residency::Handle,
}

impl RedbGraphStore {
    /// Open the store at `path`, creating an empty one — and any missing
    /// parent directory — if it does not exist.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(path).map_err(redb::Error::from)?;
        // Create the table up front so a read against a brand-new store sees
        // an empty table rather than a "does not exist" error.
        let write = db.begin_write().map_err(redb::Error::from)?;
        {
            write.open_table(EDGES).map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;

        // Size after creation, so a brand-new store registers its real (small)
        // footprint rather than zero.
        let bytes = std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        let _resident = telividb_telemetry::residency::register(
            telividb_telemetry::residency::ResidentKind::GraphStore,
            telividb_telemetry::residency::Location::Host,
            path.display().to_string(),
            bytes,
        );
        logger::debug!("graph opened").with_data(&serde_json::json!({
            fields::STORE: "graph",
            fields::BACKEND: "redb",
            fields::RESIDENT_BYTES: bytes,
        }));

        Ok(Self { db, _resident })
    }

    /// Persist one edge.
    ///
    /// Refuses an edge type containing a NUL byte. The key format is
    /// NUL-delimited, so such a type would split into the wrong fields on the
    /// way back out — and the write would have reported success for an edge
    /// that can never be read again.
    pub fn insert_edge(&self, edge: &Edge) -> Result<()> {
        if edge.edge_type.contains('\0') {
            return Err(telividb_core::Error::GraphStore {
                reason: format!(
                    "edge type {:?} contains a NUL byte, which is the key separator",
                    edge.edge_type
                ),
            }
            .into());
        }
        let key = encode_key(edge);
        let value = edge.weight.to_le_bytes();
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        {
            let mut table = write.open_table(EDGES).map_err(redb::Error::from)?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(redb::Error::from)?;
        }
        write.commit().map_err(redb::Error::from)?;
        Ok(())
    }
}

impl GraphStore for RedbGraphStore {
    #[allow(clippy::type_complexity)]
    fn iter_edges(
        &self,
    ) -> telividb_core::Result<Box<dyn Iterator<Item = telividb_core::Result<Edge>> + '_>> {
        let read = self.db.begin_read().map_err(redb_err)?;
        let table = read.open_table(EDGES).map_err(redb_err)?;

        // Rehydration is a one-time bulk load (CLAUDE.md rule 47), not a
        // lazy per-row scan interleaved with other work, so collecting here
        // avoids a self-referential iterator borrowing the read transaction.
        let mut edges = Vec::new();
        let range = table.range::<&str>(..).map_err(redb_err)?;
        for row in range {
            let (key, value) = match row {
                Ok(row) => row,
                Err(e) => {
                    edges.push(Err(redb_err(e)));
                    continue;
                }
            };
            edges.push(decode_row(key.value(), value.value()));
        }
        Ok(Box::new(edges.into_iter()))
    }
}

/// Fold any of `redb`'s operation-specific errors into the `GraphStore`
/// port's error type, by message.
///
/// `telividb-core` cannot depend on `telividb-storage` or `redb` (rule 14:
/// dependencies point inward), so there is no `From` conversion the other
/// direction — this crosses the boundary explicitly, the one place it's
/// needed, rather than growing the domain error enum a variant per adapter.
fn redb_err<E: Into<redb::Error>>(e: E) -> telividb_core::Error {
    graph_store_err(e.into().to_string())
}

fn encode_key(edge: &Edge) -> String {
    format!(
        "{}\0{}\0{}",
        edge.src.as_str(),
        edge.edge_type,
        edge.dst.as_str()
    )
}

fn decode_row(key: &str, value: &[u8]) -> telividb_core::Result<Edge> {
    let mut parts = key.splitn(3, '\0');
    let (Some(src), Some(edge_type), Some(dst)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(graph_store_err(format!(
            "malformed edge key {key:?}: expected 3 NUL-separated fields"
        )));
    };
    let src = parse_name(src)?;
    let dst = parse_name(dst)?;
    let weight = value
        .try_into()
        .map(f32::from_le_bytes)
        .map_err(|_| graph_store_err(format!("edge weight for {key:?} is not 4 bytes")))?;
    Ok(Edge::new(src, dst, edge_type.to_owned(), weight))
}

fn parse_name(raw: &str) -> telividb_core::Result<ResourceName> {
    ResourceName::parse(raw).map_err(|e| graph_store_err(e.to_string()))
}

fn graph_store_err(reason: String) -> telividb_core::Error {
    telividb_core::Error::GraphStore { reason }
}

#[cfg(test)]
#[path = "redb_graph_store_test.rs"]
mod tests;
