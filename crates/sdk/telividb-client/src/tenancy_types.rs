//! Organizations, projects and spaces, as a caller sees them.
//!
//! Plain types rather than the wire messages, for the same reason the rest of
//! this SDK converts: a caller should not have to know that a timestamp is a
//! `prost` message or that absence is an `Option` of one.

use telividb_buffers::protobuf::tenancy::v1 as wire;

/// How a space is protected, which decides what may be done with its contents.
///
/// Declared when the space is created and never changed after: protection
/// decides segment routing, so altering it later would mean rewriting every
/// segment the space owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protection {
    /// Visible according to ordinary role grants on the containing projects.
    ///
    /// Named for the wire value (`PROTECTION_NONE`) rather than "open", so the
    /// same word means the same thing in the proto, here, and in the window.
    None,
    /// Restricted by an owner predicate. An ACL, not a cryptographic guarantee.
    Private,
    /// Key-wrapped. The engine holds the key.
    Vault,
    /// Key-wrapped with a key the client holds. The engine cannot open it alone.
    Sealed,
}

impl Protection {
    /// The wire value for this protection.
    pub fn as_wire(self) -> i32 {
        match self {
            Self::None => wire::Protection::None as i32,
            Self::Private => wire::Protection::Private as i32,
            Self::Vault => wire::Protection::Vault as i32,
            Self::Sealed => wire::Protection::Sealed as i32,
        }
    }

    /// Read a wire value, treating anything unrecognised as the safest option.
    ///
    /// Fail-secure rather than fail-open: a value this build does not know is
    /// more likely a protection added later than a corrupt byte, and reading it
    /// as unprotected would expose contents the writer meant to restrict. That
    /// includes `UNSPECIFIED`, which is never valid in a response.
    pub fn from_wire(value: i32) -> Self {
        match wire::Protection::try_from(value) {
            Ok(wire::Protection::None) => Self::None,
            Ok(wire::Protection::Private) => Self::Private,
            Ok(wire::Protection::Vault) => Self::Vault,
            _ => Self::Sealed,
        }
    }

    /// The name used in configuration and across the IPC boundary.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Private => "private",
            Self::Vault => "vault",
            Self::Sealed => "sealed",
        }
    }
}

/// A tenant: the top of the resource hierarchy.
#[derive(Debug, Clone)]
pub struct Organization {
    /// Resource name, `organizations/{organization}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// How many projects it holds.
    pub project_count: i32,
    /// How many spaces it holds.
    pub space_count: i32,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<wire::Organization> for Organization {
    fn from(o: wire::Organization) -> Self {
        Self {
            name: o.name,
            display_name: o.display_name,
            project_count: o.project_count,
            space_count: o.space_count,
            deleted: o.delete_time.is_some(),
        }
    }
}

/// A unit of work inside an organization.
#[derive(Debug, Clone)]
pub struct Project {
    /// Resource name, `organizations/{organization}/projects/{project}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<wire::Project> for Project {
    fn from(p: wire::Project) -> Self {
        Self {
            name: p.name,
            display_name: p.display_name,
            deleted: p.delete_time.is_some(),
        }
    }
}

/// A protection boundary, which may span several projects.
///
/// Note it is a *sibling* of a project rather than a child: the resource name is
/// `organizations/{organization}/spaces/{space}`, and the projects it serves are
/// references. A space is where protection lives, and protection does not follow
/// the work breakdown.
#[derive(Debug, Clone)]
pub struct Space {
    /// Resource name, `organizations/{organization}/spaces/{space}`.
    pub name: String,
    /// What a person calls it.
    pub display_name: String,
    /// Projects this space serves, by resource name.
    pub projects: Vec<String>,
    /// How it is protected. Fixed at creation.
    pub protection: Protection,
    /// Whether its key is currently unavailable.
    pub locked: bool,
    /// Whether it is soft-deleted and awaiting purge.
    pub deleted: bool,
}

impl From<wire::Space> for Space {
    fn from(s: wire::Space) -> Self {
        Self {
            name: s.name,
            display_name: s.display_name,
            projects: s.projects,
            protection: Protection::from_wire(s.protection),
            locked: s.locked,
            deleted: s.delete_time.is_some(),
        }
    }
}
