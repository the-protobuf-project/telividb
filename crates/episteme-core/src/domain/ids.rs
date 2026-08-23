//! Identity types.
//!
//! The distinction here is load-bearing: [`ExternalId`] is the *only* portable
//! identity. [`Ordinal`] is segment-local and must never cross a process
//! boundary — not into an archive, not into an API response, not onto either
//! end of an exported edge. See ARCHITECTURE.md §7 and CLAUDE.md invariant 9.

/// A caller-supplied identity, stable across export, import and re-sharding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalId(pub u64);

/// A row position *within one segment*. Meaningless anywhere else.
///
/// Deliberately not `pub`-constructible from a bare integer outside this crate,
/// so an ordinal cannot be casually minted where an [`ExternalId`] was meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ordinal(u32);

impl Ordinal {
    /// Only storage constructs ordinals, and only while reading a segment.
    pub fn from_row(row: u32) -> Self {
        Self(row)
    }

    pub fn row(self) -> u32 {
        self.0
    }
}

/// The width of a named vector field. Non-zero by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dim(u32);

impl Dim {
    pub fn new(dim: u32) -> crate::Result<Self> {
        if dim == 0 {
            return Err(crate::Error::ZeroDim);
        }
        Ok(Self(dim))
    }

    pub fn get(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
#[path = "ids_test.rs"]
mod tests;
