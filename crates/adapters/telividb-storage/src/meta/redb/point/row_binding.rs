//! Which point a vector row belongs to.
//!
//! Split from `redb_point_store.rs` because it answers a different question:
//! that file stores points, this maps a *search hit* back to one.
//!
//! The mapping exists because a hit is a segment-local ordinal (invariant 9),
//! meaningless outside the segment that produced it. `VectorField::row_of`
//! widens that to a field-wide row; this turns the row into the only identity
//! allowed to cross a process boundary — the point's resource name.

use super::store::{ROWS, RedbPointStore};
use crate::error::Result;
use redb::ReadableDatabase;
use telividb_core::ResourceName;

impl RedbPointStore {
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
        // A present-but-unparsable binding is corruption, not absence.
        // Reporting `None` would silently drop a real hit from a search.
        match table.get((field, row)).map_err(redb::Error::from)? {
            Some(v) => Ok(Some(ResourceName::parse(v.value()).map_err(|e| {
                telividb_core::Error::PointStore {
                    reason: format!("row binding for {field}:{row} is malformed: {e}"),
                }
            })?)),
            None => Ok(None),
        }
    }
}
