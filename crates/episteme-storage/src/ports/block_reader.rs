//! How bytes are fetched from a sealed segment.

use crate::Result;

/// Random access to the bytes of one sealed segment file.
///
/// This exists so the read path is not welded to `mmap`. Memory mapping is the
/// right default for immutable, read-mostly segments, but it has real costs —
/// page faults block the calling thread and eviction is the kernel's decision,
/// so tail latency under memory pressure is not ours to control. Keeping the
/// seam here means a direct-IO or io_uring reader can replace it later without
/// touching a single index.
pub trait BlockReader: Send + Sync {
    /// Total length of the underlying file.
    fn len(&self) -> u64;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Read exactly `buf.len()` bytes starting at `offset`.
    ///
    /// Implementations must either fill `buf` completely or fail; a short read
    /// is an error, never a silent partial result.
    fn read_exact_at(&self, offset: u64, buf: &mut [u8]) -> Result<()>;
}
