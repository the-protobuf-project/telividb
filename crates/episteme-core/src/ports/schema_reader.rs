//! Resolving descriptor bytes into a domain schema.

use crate::{CollectionSchema, Fingerprint, Result};

/// Turns a serialized `FileDescriptorSet` into a [`CollectionSchema`].
///
/// **The engine never parses `.proto`.** A generator produces the descriptor
/// set; `CreateCollection` carries those bytes; they are stored verbatim in
/// `meta.redb` and are the authoritative schema. An adapter — `prost-reflect`
/// today — resolves them into pure domain types on the way in.
///
/// This is a port rather than a core type because descriptor reflection is
/// I/O-shaped and version-bound. Keeping it behind a trait keeps `prost-reflect`
/// out of the planner and the index, and leaves room for a second schema source
/// without touching either.
pub trait SchemaReader: Send + Sync {
    /// Resolve descriptor bytes for one collection.
    ///
    /// Descriptor sets arriving over an RPC are **untrusted input** — an
    /// implementation must bound recursion and reject malformed input rather
    /// than trusting the producer.
    fn resolve(&self, collection: &str, descriptor_set: &[u8]) -> Result<CollectionSchema>;

    /// Fingerprint descriptor bytes without fully resolving them.
    ///
    /// Must hash the **canonicalized** form, so that a descriptor set differing
    /// only in field ordering or serialization details produces the same digest.
    /// Otherwise a harmless re-encode reads as schema drift and every segment
    /// written before it becomes unreadable.
    fn fingerprint(&self, descriptor_set: &[u8]) -> Fingerprint;
}
