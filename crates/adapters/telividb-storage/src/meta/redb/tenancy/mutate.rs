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

use super::store::{ORGANIZATIONS, PROJECTS, RETENTION_MILLIS, RedbTenancyStore, SESSIONS, SPACES};
use super::{organization, project, session, space};
use crate::error::Result;
use redb::{ReadableTable, TableDefinition};
use telividb_core::{Lifecycle, Organization, Project, ResourceName, Session, Space};

impl RedbTenancyStore {
    /// Tombstone an organization, leaving it recoverable until it expires.
    pub fn delete_organization(
        &self,
        name: &ResourceName,
        now_millis: i64,
    ) -> Result<Option<Organization>> {
        self.tombstone(
            ORGANIZATIONS,
            name,
            now_millis,
            organization::decode,
            organization::encode,
            |o| &mut o.lifecycle,
        )
    }

    /// Restore a tombstoned organization.
    pub fn undelete_organization(
        &self,
        name: &ResourceName,
        now_millis: i64,
    ) -> Result<Option<Organization>> {
        self.restore(
            ORGANIZATIONS,
            name,
            now_millis,
            organization::decode,
            organization::encode,
            |o| &mut o.lifecycle,
        )
    }

    /// Tombstone a project.
    pub fn delete_project(&self, name: &ResourceName, now: i64) -> Result<Option<Project>> {
        self.tombstone(PROJECTS, name, now, project::decode, project::encode, |p| {
            &mut p.lifecycle
        })
    }

    /// Restore a tombstoned project.
    pub fn undelete_project(&self, name: &ResourceName, now: i64) -> Result<Option<Project>> {
        self.restore(PROJECTS, name, now, project::decode, project::encode, |p| {
            &mut p.lifecycle
        })
    }

    /// Tombstone a space.
    pub fn delete_space(&self, name: &ResourceName, now: i64) -> Result<Option<Space>> {
        self.tombstone(SPACES, name, now, space::decode, space::encode, |s| {
            &mut s.lifecycle
        })
    }

    /// Restore a tombstoned space.
    pub fn undelete_space(&self, name: &ResourceName, now: i64) -> Result<Option<Space>> {
        self.restore(SPACES, name, now, space::decode, space::encode, |s| {
            &mut s.lifecycle
        })
    }

    /// Tombstone a session.
    pub fn delete_session(&self, name: &ResourceName, now: i64) -> Result<Option<Session>> {
        self.tombstone(SESSIONS, name, now, session::decode, session::encode, |s| {
            &mut s.lifecycle
        })
    }

    /// Restore a tombstoned session.
    pub fn undelete_session(&self, name: &ResourceName, now: i64) -> Result<Option<Session>> {
        self.restore(SESSIONS, name, now, session::decode, session::encode, |s| {
            &mut s.lifecycle
        })
    }

    /// Stamp a tombstone and an expiry, if the resource is live.
    ///
    /// Returns the tombstoned resource, or `None` if there was nothing live to
    /// delete. Deleting twice is not an error and not a second stamp: moving
    /// the expiry would quietly extend the life of something the caller
    /// believes it already deleted.
    fn tombstone<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        name: &ResourceName,
        now_millis: i64,
        decode: fn(ResourceName, &[u8]) -> Result<T>,
        encode: fn(&T) -> Vec<u8>,
        lifecycle: fn(&mut T) -> &mut Lifecycle,
    ) -> Result<Option<T>> {
        self.amend(table, name, decode, encode, |value| {
            let life = lifecycle(value);
            if !life.is_live() {
                return false;
            }
            life.deleted_at = Some(now_millis);
            life.expires_at = Some(now_millis + RETENTION_MILLIS);
            life.updated_at = now_millis;
            true
        })
    }

    /// Clear a tombstone, if there is one.
    ///
    /// `None` when there is nothing to restore — either no such resource, or
    /// one that was never deleted. Both are the caller's to report, and neither
    /// is an error here.
    fn restore<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        name: &ResourceName,
        now_millis: i64,
        decode: fn(ResourceName, &[u8]) -> Result<T>,
        encode: fn(&T) -> Vec<u8>,
        lifecycle: fn(&mut T) -> &mut Lifecycle,
    ) -> Result<Option<T>> {
        self.amend(table, name, decode, encode, |value| {
            let life = lifecycle(value);
            if life.is_live() {
                return false;
            }
            life.deleted_at = None;
            life.expires_at = None;
            life.updated_at = now_millis;
            true
        })
    }

    /// Read one row, let `change` amend it, and write it back.
    ///
    /// The read and the write share a transaction, so two windows deleting the
    /// same resource cannot both observe it live and both stamp it.
    fn amend<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        name: &ResourceName,
        decode: fn(ResourceName, &[u8]) -> Result<T>,
        encode: fn(&T) -> Vec<u8>,
        change: impl FnOnce(&mut T) -> bool,
    ) -> Result<Option<T>> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let changed = {
            let mut open = write.open_table(table).map_err(redb::Error::from)?;
            // Decoded and the borrow released before the insert: `redb` holds a
            // read guard on the value, and writing through the same table while
            // it lives would borrow the table twice.
            let decoded = match open.get(name.as_str()).map_err(redb::Error::from)? {
                Some(value) => Some(decode(name.clone(), value.value())?),
                None => None,
            };
            match decoded {
                Some(mut value) => {
                    if change(&mut value) {
                        open.insert(name.as_str(), encode(&value).as_slice())
                            .map_err(redb::Error::from)?;
                        Some(value)
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
