//! Tombstoning a tenancy resource, and bringing one back.
//!
//! Split from the store because it is a different operation from reading and
//! creating: every method here reads a record, amends its lifecycle, and writes
//! it back inside one transaction — so two windows deleting the same
//! organization cannot both observe it live and both stamp it.
//!
//! Nothing here removes a row. That is what makes `undelete` possible, and it
//! is why the protos carry `delete_time` and `expire_time` on every resource in
//! this tree rather than treating deletion as an absence.

use super::record::{decode, encode};
use super::store::{ORGANIZATIONS, RETENTION_MILLIS, RedbTenancyStore};
use crate::error::Result;
use redb::ReadableTable;
use telividb_core::{Organization, ResourceName};

impl RedbTenancyStore {
    /// Tombstone an organization, leaving it recoverable until it expires.
    ///
    /// Returns the tombstoned organization, or `None` if there was nothing live
    /// to delete. Returning the resource rather than a bare success is what
    /// lets a caller report what it did — and it is the soft-delete form the
    /// API asks for.
    pub fn delete_organization(
        &self,
        name: &ResourceName,
        now_millis: i64,
    ) -> Result<Option<Organization>> {
        self.restamp(name, |org| {
            if !org.lifecycle.is_live() {
                return false;
            }
            org.lifecycle.deleted_at = Some(now_millis);
            org.lifecycle.expires_at = Some(now_millis + RETENTION_MILLIS);
            org.lifecycle.updated_at = now_millis;
            true
        })
    }

    /// Restore a tombstoned organization.
    ///
    /// Returns `None` when there is nothing to restore — either no such
    /// organization, or one that was never deleted. Both are the caller's to
    /// report, and neither is an error here.
    pub fn undelete_organization(
        &self,
        name: &ResourceName,
        now_millis: i64,
    ) -> Result<Option<Organization>> {
        self.restamp(name, |org| {
            if org.lifecycle.is_live() {
                return false;
            }
            org.lifecycle.deleted_at = None;
            org.lifecycle.expires_at = None;
            org.lifecycle.updated_at = now_millis;
            true
        })
    }

    /// Read one organization, let `change` amend it, and write it back.
    ///
    /// The read and the write share a transaction, so two windows deleting the
    /// same organization cannot both observe it live and both stamp it.
    fn restamp(
        &self,
        name: &ResourceName,
        change: impl FnOnce(&mut Organization) -> bool,
    ) -> Result<Option<Organization>> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let changed = {
            let mut table = write.open_table(ORGANIZATIONS).map_err(redb::Error::from)?;
            // Decoded and the borrow released before the insert: `redb` holds a
            // read guard on the value, and writing through the same table while
            // it lives would borrow the table twice.
            let decoded = match table.get(name.as_str()).map_err(redb::Error::from)? {
                Some(value) => Some(decode(name.clone(), value.value())?),
                None => None,
            };
            // A pattern guard cannot borrow mutably, so the amendment happens
            // in the body rather than in the match.
            match decoded {
                Some(mut org) => {
                    if change(&mut org) {
                        table
                            .insert(name.as_str(), encode(&org).as_slice())
                            .map_err(redb::Error::from)?;
                        Some(org)
                    } else {
                        None
                    }
                }
                None => None,
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(changed)
    }
}
