//! Projects, spaces and sessions: everything under an organization.
//!
//! Split from the organization's own methods because the organization is the
//! one resource that is not contained by anything — it is the root, and the
//! three here all live beneath it. They also share a shape the root does not:
//! each is created, read, listed and tombstoned identically, differing only in
//! which table and which codec.

use super::store::{PROJECTS, RedbTenancyStore, SESSIONS, SPACES};
use super::{project, session, space};
use crate::error::Result;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use telividb_core::{Project, ResourceName, Session, Space};

impl RedbTenancyStore {
    /// Persist a new project. `false` if one already holds that name.
    pub fn create_project(&self, value: &Project) -> Result<bool> {
        self.insert(PROJECTS, &value.name, project::encode(value))
    }

    /// Fetch one project, including a tombstoned one.
    pub fn project(&self, name: &ResourceName) -> Result<Option<Project>> {
        self.fetch(PROJECTS, name, project::decode)
    }

    /// Projects, in resource-name order.
    pub fn projects(&self, show_deleted: bool) -> Result<Vec<Project>> {
        let all = self.fetch_all(PROJECTS, project::decode)?;
        Ok(filter(all, show_deleted, |p| p.lifecycle.is_live()))
    }

    /// Persist a new space. `false` if one already holds that name.
    pub fn create_space(&self, value: &Space) -> Result<bool> {
        self.insert(SPACES, &value.name, space::encode(value))
    }

    /// Fetch one space, including a tombstoned one.
    pub fn space(&self, name: &ResourceName) -> Result<Option<Space>> {
        self.fetch(SPACES, name, space::decode)
    }

    /// Spaces, in resource-name order.
    pub fn spaces(&self, show_deleted: bool) -> Result<Vec<Space>> {
        let all = self.fetch_all(SPACES, space::decode)?;
        Ok(filter(all, show_deleted, |s| s.lifecycle.is_live()))
    }

    /// Persist a new session. `false` if one already holds that name.
    pub fn create_session(&self, value: &Session) -> Result<bool> {
        self.insert(SESSIONS, &value.name, session::encode(value))
    }

    /// Fetch one session, including a tombstoned one.
    pub fn session(&self, name: &ResourceName) -> Result<Option<Session>> {
        self.fetch(SESSIONS, name, session::decode)
    }

    /// Sessions, in resource-name order.
    pub fn sessions(&self, show_deleted: bool) -> Result<Vec<Session>> {
        let all = self.fetch_all(SESSIONS, session::decode)?;
        Ok(filter(all, show_deleted, |s| s.lifecycle.is_live()))
    }

    /// Insert if the name is free. `false` without writing if it is taken.
    ///
    /// `Create`, not `Upsert` — and a tombstoned resource still holds its name,
    /// so creating over one is refused too. The name is taken until it expires,
    /// and reusing it would put a new resource under an identity that archives
    /// and edges still point at.
    fn insert(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        name: &ResourceName,
        bytes: Vec<u8>,
    ) -> Result<bool> {
        let write = self.db.begin_write().map_err(redb::Error::from)?;
        let created = {
            let mut open = write.open_table(table).map_err(redb::Error::from)?;
            let key = name.as_str();
            if open.get(key).map_err(redb::Error::from)?.is_some() {
                false
            } else {
                open.insert(key, bytes.as_slice())
                    .map_err(redb::Error::from)?;
                true
            }
        };
        write.commit().map_err(redb::Error::from)?;
        Ok(created)
    }

    /// Read and decode one row.
    fn fetch<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        name: &ResourceName,
        decode: fn(ResourceName, &[u8]) -> Result<T>,
    ) -> Result<Option<T>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let open = read.open_table(table).map_err(redb::Error::from)?;
        match open.get(name.as_str()).map_err(redb::Error::from)? {
            Some(value) => Ok(Some(decode(name.clone(), value.value())?)),
            None => Ok(None),
        }
    }

    /// Read and decode every row, in key order.
    fn fetch_all<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        decode: fn(ResourceName, &[u8]) -> Result<T>,
    ) -> Result<Vec<T>> {
        let read = self.db.begin_read().map_err(redb::Error::from)?;
        let open = read.open_table(table).map_err(redb::Error::from)?;

        let mut out = Vec::new();
        for row in open.iter().map_err(redb::Error::from)? {
            let (key, value) = row.map_err(redb::Error::from)?;
            let name = ResourceName::parse(key.value()).map_err(|e| {
                crate::error::Error::Domain(telividb_core::Error::PointStore {
                    reason: e.to_string(),
                })
            })?;
            out.push(decode(name, value.value())?);
        }
        Ok(out)
    }
}

/// Drop tombstones unless the caller asked for them.
fn filter<T>(all: Vec<T>, show_deleted: bool, live: impl Fn(&T) -> bool) -> Vec<T> {
    match show_deleted {
        true => all,
        false => all.into_iter().filter(|item| live(item)).collect(),
    }
}

#[cfg(test)]
#[path = "children_test.rs"]
mod tests;
