//! A validated resource name.

use crate::error::{Error, Result};

/// A slash-separated resource name, e.g. `collections/finance/points/doc-123`.
///
/// Validated on construction: no empty segments, no leading or trailing slash.
/// Segments are otherwise unrestricted, so ids containing `.`, `-` or `_` are
/// fine — but never `/`, which is the separator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn parse(raw: impl Into<String>) -> Result<Self> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(Error::InvalidResourceName {
                name: raw,
                reason: "must not be empty",
            });
        }
        if raw.starts_with('/') || raw.ends_with('/') {
            return Err(Error::InvalidResourceName {
                name: raw,
                reason: "must not begin or end with '/'",
            });
        }
        if raw.split('/').any(str::is_empty) {
            return Err(Error::InvalidResourceName {
                name: raw,
                reason: "must not contain an empty segment",
            });
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Segments, in order.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The final segment — the id within its collection type.
    ///
    /// `collections/finance/points/doc-123` yields `doc-123`.
    pub fn leaf(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// The name with its last two segments removed — the containing resource.
    ///
    /// `collections/finance/points/doc-123` yields `collections/finance`.
    /// Returns `None` when there is no parent to speak of.
    pub fn parent(&self) -> Option<ResourceName> {
        let (parent, _) = self.0.rsplit_once('/')?;
        let (parent, _) = parent.rsplit_once('/')?;
        Some(Self(parent.to_owned()))
    }

    /// Whether this name lies under `pattern`, where `*` matches exactly one
    /// segment and a trailing `**` matches one or more remaining segments.
    ///
    /// This is what makes a grant like `collections/finance/points/*` express
    /// authorization scope directly. See ARCHITECTURE.md §6.
    pub fn matches(&self, pattern: &str) -> bool {
        let mut want = pattern.split('/');
        let mut have = self.0.split('/');

        loop {
            match (want.next(), have.next()) {
                (Some("**"), Some(_)) => return true,
                (Some(w), Some(h)) if w == "*" || w == h => continue,
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

impl std::fmt::Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
#[path = "name_test.rs"]
mod tests;
