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
        progress: &mut dyn FnMut(u64),
    ) -> Result<()> {
        self.offsets.lock().expect("not poisoned").push(offset);
        sink.write_all(&self.body[at(&self.body, offset)..])?;
        progress(self.body.len() as u64);
        Ok(())
    }
}
