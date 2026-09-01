//! Who a caller is, and what they were granted.
//!
//! # These are records, and nothing enforces them yet
//!
//! `telividb-policy` does not exist, so a role binding here is a row that no
//! query consults. Six invariants (15, 21, 34, 35, 36, 44) describe an
//! authorization system with nothing behind it, and storing a grant does not
//! begin to implement one.
//!
//! That is worth stating in the type's own documentation rather than only in a
//! design note, because the failure mode is specific: a screen that lists users
//! and roles reads as a working permission system, and someone could reasonably
//! conclude their data is protected by it. It is not. These types exist so the
//! shape is settled before the engine that reads them is written.
//!
//! # A user is not a principal
//!
//! [`User`] is the record a person administers — a display name, group
//! membership, something to click. `principal` is the identity an authenticated
//! request actually arrives with, and the two are separate because the second
//! comes from whatever issued the credential and cannot be renamed from here.

use super::tenancy::Lifecycle;
use crate::ResourceName;

/// Someone who can be granted access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Resource name, `users/{user}`.
    ///
    /// Not scoped to an organization: a person may hold grants in several, and
    /// duplicating them per tenant would make "the same person" a matter of
    /// string comparison across trees.
    pub name: ResourceName,
    /// What a person calls this user.
    pub display_name: String,
    /// The identity an authenticated request arrives with.
    ///
    /// Issued by whatever authenticated the caller, not by this system — so it
    /// is carried rather than assigned, and it is what a future policy engine
    /// would match on. Empty for a user that exists as a record only.
    pub principal: String,
    /// Groups this user belongs to, by resource name.
    ///
    /// Membership rather than a name segment: a user moves between groups, and a
    /// move that renamed the user would break every grant pointing at them.
    pub user_groups: Vec<ResourceName>,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}

/// A named set of users, and what a role is actually granted to.
///
/// Grants attach to groups rather than to users so that removing someone's
/// access is one membership change rather than a search for every binding that
/// happens to name them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGroup {
    /// Resource name, `userGroups/{user_group}`.
    pub name: ResourceName,
    /// What a person calls this group — "Engineering", "Finance".
    pub display_name: String,
    /// Creation, modification and deletion times.
    pub lifecycle: Lifecycle,
}

/// A role, granted to a group, over some scope.
///
/// **Recorded, not enforced.** Nothing reads this when answering a query. See
/// this module's own documentation for why that is stated rather than assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleBinding {
    /// Resource name, `organizations/{organization}/roleBindings/{role_binding}`.
    pub name: ResourceName,
    /// The group this grants to, by resource name.
    pub user_group: ResourceName,
    /// The role granted, e.g. `roles/reader`.
    pub role: String,
    /// What the grant covers: an organization, a project, or a space.
    ///
    /// A resource name rather than a type plus an id, so a binding over a
    /// project and one over a space are the same shape — which is what lets a
    /// future policy engine resolve them with one lookup instead of three.
    pub scope: ResourceName,
    /// When this binding was created.
    ///
    /// A binding is created and revoked rather than edited: changing what a
    /// grant covers is a different grant, and an audit trail that let one turn
    /// into another silently would not be one.
    pub created_at: i64,
}
