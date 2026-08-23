//! Content fingerprints for schema and model provenance.
//!
//! "Self-describing" has to mean something stronger than "parseable". A segment
//! names the exact schema it was written under and the exact model that
//! produced each vector field, so a segment copied to another machine can be
//! **validated** rather than merely read. That property is what makes archives
//! and replication safe instead of hopeful.
//!
//! Two failures this prevents, both of which are silent otherwise:
//!
//! - A segment written under a drifted `.proto` read back as though the schema
//!   had not changed — columns land in the wrong place and nothing errors.
//! - Vectors from a different embedding model merged into one index. Nothing
//!   fails; recall simply degrades, and every neighbour returned is plausible
//!   and wrong.

/// A 32-byte SHA-256 digest identifying a schema descriptor set or a model file.
///
/// SHA-256 rather than a faster hash for two reasons: it is what §5.3 already
/// specifies for content references, so the system has one hash family rather
/// than three; and it is available in pure Rust with no C build dependency,
/// which the toolchain-free build rule requires.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fingerprint([u8; 32]);

impl Fingerprint {
    pub const BYTES: usize = 32;

    /// Fingerprint arbitrary bytes — a serialized descriptor set, or a GGUF file.
    pub fn of(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(hasher.finalize().into())
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// All-zero fingerprint, meaning "not recorded".
    ///
    /// Distinguishable from a real digest in practice, and used by fixtures and
    /// by segments written before a schema was bound. Reading one back is a
    /// signal to skip validation, never to assume agreement.
    pub const fn unset() -> Self {
        Self([0u8; 32])
    }

    pub fn is_unset(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Short prefix for logs and error messages.
    ///
    /// Twelve hex characters — enough to tell two fingerprints apart at a
    /// glance, short enough to read in a log line.
    pub fn short(&self) -> String {
        self.0[..6].iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl std::fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fingerprint({})", self.short())
    }
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.short())
    }
}

#[cfg(test)]
#[path = "fingerprint_test.rs"]
mod tests;
