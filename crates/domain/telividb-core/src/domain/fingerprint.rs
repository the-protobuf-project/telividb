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
    /// Digest width in bytes.
    pub const BYTES: usize = 32;

    /// Fingerprint arbitrary bytes — a serialized descriptor set, or a GGUF file.
    pub fn of(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(hasher.finalize().into())
    }

    /// Fingerprint a stream, without holding it in memory.
    ///
    /// The counterpart to [`of`](Self::of), and the one to reach for on a file.
    /// A model is hundreds of megabytes and the catalog checks every installed
    /// one each time it is listed — reading them whole would allocate gigabytes
    /// to answer a question about a directory.
    ///
    /// The buffer is 64 KiB: large enough that the syscall cost disappears,
    /// small enough to stay off the heap's slow path.
    pub fn of_reader(mut source: impl std::io::Read) -> std::io::Result<Self> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let read = source.read(&mut buffer)?;
            if read == 0 {
                return Ok(Self(hasher.finalize().into()));
            }
            hasher.update(&buffer[..read]);
        }
    }

    /// Wrap a digest already computed elsewhere, such as one read from a header.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw digest, for writing into a header.
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

    /// Whether this is the all-zero "not recorded" fingerprint.
    pub fn is_unset(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// The full 64-character lowercase hex digest.
    ///
    /// [`Display`](std::fmt::Display) deliberately prints the short form
    /// instead, because a log line wants twelve characters. This is the form
    /// that goes into a catalog file or gets compared against a model host's
    /// API, where the whole digest is the point.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Parse a 64-character hex digest.
    ///
    /// Returns `None` for anything that is not exactly 64 hex characters,
    /// rather than accepting a short or malformed digest and silently
    /// comparing against a value that can never match — which would present as
    /// a corrupt download rather than as the typo it is.
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != Self::BYTES * 2 {
            return None;
        }
        let mut out = [0u8; Self::BYTES];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
        }
        Some(Self(out))
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
