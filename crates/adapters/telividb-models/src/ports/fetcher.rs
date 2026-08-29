//! The one place this crate touches the network.

use crate::Result;

/// Fetches bytes from a URL.
///
/// A port rather than a direct HTTP call, for three reasons that all turned out
/// to matter. Everything above it — the catalog, the architecture gate, digest
/// verification, resume — is testable with no network and no fixture server.
/// The TLS stack stays optional, which keeps this crate publishable on its own
/// (rule 51) and keeps a native dependency out of the default build (rule 1).
/// And a caller that would rather use its own client, or a proxy, or a local
/// mirror, implements this instead of being told no.
///
/// Synchronous on purpose. This crate is embeddable and runtime-agnostic, like
/// storage and index; `tokio` belongs to the server (see *Async* in CLAUDE.md).
pub trait Fetcher: Send + Sync {
    /// Read `len` bytes from `offset`.
    ///
    /// This is what makes judging a model cheap: the header sits in the first
    /// couple of megabytes, so an architecture can be checked with a range
    /// request rather than by downloading several hundred megabytes and
    /// discovering it does not load.
    ///
    /// A host that ignores the range and answers with the whole file is not an
    /// error — the caller takes the prefix it asked for.
    fn range(&self, url: &str, offset: u64, len: u64) -> Result<Vec<u8>>;

    /// Fetch a whole resource as text.
    ///
    /// For API responses, which are small. Never for a model file.
    fn text(&self, url: &str) -> Result<String>;

    /// Stream a resource from `offset` into `sink`, reporting cumulative
    /// progress as it goes.
    ///
    /// `offset` is what makes a download resumable: an interrupted install
    /// leaves a partial file, and the next attempt continues from its length
    /// rather than starting again. Invariant 10 asks for exactly this, and a
    /// model file is large enough that the difference is the product working
    /// on a bad connection or not.
    ///
    /// `progress` is called with the total bytes written so far, not a delta.
    fn stream(
        &self,
        url: &str,
        offset: u64,
        sink: &mut dyn std::io::Write,
        progress: &mut dyn FnMut(u64),
    ) -> Result<()>;
}
