//! The tenancy tree: organizations, projects, spaces and sessions.
//!
//! Only the organization appears in a resource name. Project, space and session
//! are membership carried as fields, because a conversation moves between them
//! and a move that renamed the resource would break every stored reference to
//! it — external names are the only portable identity there is.

use crate::ResourceName;

/// How the contents of a space are protected.
///
/// The distinction is load-bearing rather than descriptive: only the last two
/// involve cryptography, and calling an access-controlled space a vault invites
/// the assumption that it survives a compromised server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Protection {
    /// Visible according to ordinary role grants on the containing projects.
    #[default]
    None,
    /// Readable only by its owner, enforced by a visibility predicate.
    ///
    /// Access control, not cryptography. Anyone who compromises the server
    /// reads it, which is exactly why it is not called a vault.
    Private,
    /// Encrypted with a key the server wraps and holds.
    ///
    /// Contents live in their own segments so unlocking decrypts a region
    /// rather than a row at a time.
    Vault,
    /// Encrypted with a key only the client holds.
    ///
    /// The server cannot read the contents even when compromised, and cannot
    /// search them while locked.
    Sealed,
}

/// When a resource was created, changed, and — if it was — deleted.
///
/// Deletion is soft everywhere in this tree: `deleted_at` set means excluded
/// from queries, and the bytes survive until expiry. That is what makes
/// `Undelete` possible, and it is why these travel together rather than as
/// three unrelated fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Lifecycle {
    /// Milliseconds since the Unix epoch when the resource was created.
    pub created_at: i64,
    /// Milliseconds since the Unix epoch when it was last modified.
    pub updated_at: i64,
    /// When it was soft-deleted, or `None` while it is live.
    pub deleted_at: Option<i64>,
    /// When a soft-deleted resource becomes unrecoverable.
    pub expires_at: Option<i64>,
}

impl Lifecycle {
    /// Whether this resource is currently visible to a query.
    pub fn is_live(&self) -> bool {
        self.deleted_at.is_none()
    }
}

/// The root of the tenancy tree, and the physical boundary.
///
/// One organization is one collection, so purging a tenant never rewrites
/// another tenant's segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    /// Resource name, `organizations/{organization}`.
    pub name: ResourceName,
    /// What a person calls this organization.
    ///
    /// Distinct from the resource name, which is an identifier and permanent.
    /// Renaming the one a reader sees must not rename the one every stored
    /// reference points at.
    pub display_name: String,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}

/// A unit of work within an organization, and where access is granted.
///
/// A project is a predicate, not a collection. At the scale this is designed
/// for, a collection per project would mean thousands of indexes over a handful
/// of conversations each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Resource name, `organizations/{organization}/projects/{project}`.
    pub name: ResourceName,
    /// What a person calls this project, and free to change.
    pub display_name: String,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}

/// A named container for related conversation, and where protection is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    /// Resource name, `organizations/{organization}/spaces/{space}`.
    pub name: ResourceName,
    /// What a person calls this space — "Finance", "Board".
    pub display_name: String,
    /// Projects this space belongs to.
    ///
    /// Repeated and mutable: a space may span projects, and membership is not
    /// in the resource name precisely so it can change without renaming
    /// anything that refers to this space.
    pub projects: Vec<ResourceName>,
    /// How the contents are protected.
    ///
    /// Fixed at creation, because it decides segment routing: an encrypted
    /// space writes into its own segments so unlocking decrypts a region rather
    /// than a row at a time. Changing it later is a rewrite, not a field
    /// update.
    pub protection: Protection,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}

/// A recorded working period holding several conversations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Resource name, `organizations/{organization}/sessions/{session}`.
    pub name: ResourceName,
    /// Human-readable name, when it has one.
    pub display_name: String,
    /// Space this session belongs to. Mutable, and therefore a field.
    pub space: Option<ResourceName>,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}
