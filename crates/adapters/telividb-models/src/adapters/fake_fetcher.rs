//! A [`Fetcher`] backed by memory, for tests.
//!
//! Shared rather than written twice, because two test modules need one and a
//! second copy is a second thing to keep honest. It is what makes resume,
//! verification and API handling testable with no server and no network.

use crate::{Fetcher, Result};
use std::sync::Mutex;

/// Serves fixed bytes and records what it was asked for.
pub(super) struct FakeFetcher {
    /// The bytes this "host" serves, for every URL alike.
    body: Vec<u8>,
    /// The offset of each `stream` call, in order, so a test can assert that a
    /// download resumed rather than started over.
    offsets: Mutex<Vec<u64>>,
}

impl FakeFetcher {
    /// Bytes per write, small so a test can stop a transfer part-way.
    const CHUNK: usize = 4;

    /// A host serving `body`.
    pub(super) fn serving(body: impl Into<Vec<u8>>) -> Self {
        Self {
            body: body.into(),
            offsets: Mutex::new(Vec::new()),
        }
    }

    /// The offsets it was asked to stream from.
    pub(super) fn offsets(&self) -> Vec<u64> {
        self.offsets.lock().expect("not poisoned").clone()
    }
}

/// Clamp a `u64` position into the body's bounds.
fn at(body: &[u8], offset: u64) -> usize {
    usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(body.len())
}

impl Fetcher for FakeFetcher {
    fn range(&self, _url: &str, offset: u64, len: u64) -> Result<Vec<u8>> {
        let start = at(&self.body, offset);
        let end = at(&self.body, offset.saturating_add(len));
        Ok(self.body[start..end].to_vec())
    }

    fn text(&self, _url: &str) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.body).into_owned())
    }

    fn stream(
        &self,
        _url: &str,
        offset: u64,
        sink: &mut dyn std::io::Write,
        progress: &mut dyn FnMut(u64) -> bool,
    ) -> Result<()> {
        self.offsets.lock().expect("not poisoned").push(offset);

        // Chunked, like the real client. Writing the whole body in one go
        // would make this fake unable to express a cancel — the transfer would
        // already be complete by the time the caller was asked whether to
        // continue — and a test double that cannot reach a state the real
        // implementation reaches is worse than none.
        let mut at = at(&self.body, offset);
        while at < self.body.len() {
            let end = (at + Self::CHUNK).min(self.body.len());
            sink.write_all(&self.body[at..end])?;
            at = end;
            if !progress(at as u64) {
                return Ok(());
            }
        }
        Ok(())
    }
}
