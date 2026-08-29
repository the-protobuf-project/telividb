//! Typed keys for the resource names this system hands out.
//!
//! # Why a template rather than string surgery
//!
//! A resource name is the only portable identity here — internal ordinals are
//! segment-local and mean nothing outside the process that produced them. So
//! the name is what an archive carries, what an edge endpoint holds, and what a
//! citation points at, and getting one wrong is not a formatting mistake.
//!
//! Each key below declares its template once. Parsing checks the shape rather
//! than assuming it, so `organizations/acme/projects/x` cannot be read as an
//! organization, and generating cannot produce a name whose segments were
//! joined in the wrong order.
//!
//! # Shared with every other language in the ecosystem
//!
//! `resourcename` implements the same AIP-122 templates for Go, Python,
//! TypeScript, Swift and C. A template written here means the same thing to a
//! client in another language, which is the point of using a library rather
//! than parsing strings locally.

use resourcename::Resource;
use serde::{Deserialize, Serialize};

/// A key for `organizations/{organization}`.
///
/// The root of the tenancy tree, and the one container that appears in the name
/// of everything beneath it — because it is the one that never changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(template = "organizations/{organization}")]
pub struct OrganizationKey {
    /// The organization's id, forming the final path segment.
    pub organization: String,
}

/// A key for `organizations/{organization}/projects/{project}`.
///
/// A project is where access is granted, so its name is what a role binding
/// points at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(template = "organizations/{organization}/projects/{project}")]
pub struct ProjectKey {
    /// The organization this project belongs to.
    pub organization: String,
    /// The project's id.
    pub project: String,
}

/// A key for `organizations/{organization}/spaces/{space}`.
///
/// A space is where protection is declared, and protection decides segment
/// routing — so this name identifies a set of segments, not just a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(template = "organizations/{organization}/spaces/{space}")]
pub struct SpaceKey {
    /// The organization this space belongs to.
    pub organization: String,
    /// The space's id.
    pub space: String,
}

/// A key for `organizations/{organization}/sessions/{session}`.
///
/// A session groups conversations into one recorded working period, which is
/// a thing a person names and returns to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(template = "organizations/{organization}/sessions/{session}")]
pub struct SessionKey {
    /// The organization this session belongs to.
    pub organization: String,
    /// The session's id.
    pub session: String,
}

/// A key for `organizations/{organization}/conversations/{conversation}`.
///
/// Named under the organization and nothing else. Session, space and project
/// are fields on the resource rather than segments here, because a conversation
/// moves between them — and a move that renamed it would break every stored
/// edge endpoint and every citation that refers to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(template = "organizations/{organization}/conversations/{conversation}")]
pub struct ConversationKey {
    /// The organization this conversation belongs to.
    pub organization: String,
    /// The conversation's id.
    pub conversation: String,
}

/// A key for a message within a conversation.
///
/// A message never moves between conversations, so this nesting is safe in a
/// way the conversation's own name is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
#[resource_name(
    template = "organizations/{organization}/conversations/{conversation}/messages/{message}"
)]
pub struct MessageKey {
    /// The organization this message belongs to.
    pub organization: String,
    /// The conversation this message is part of.
    pub conversation: String,
    /// The message's id.
    pub message: String,
}

#[cfg(test)]
#[path = "keys_test.rs"]
mod tests;
