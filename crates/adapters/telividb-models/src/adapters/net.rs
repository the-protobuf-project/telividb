//! [`Fetcher`] over the ecosystem's HTTP client.

use super::net_url;
use crate::{Error, Fetcher, Result};
use std::collections::HashMap;
use tpp_network::{ConnectionOptions, HttpClient, HttpMethod, UrlOptions, UrlScheme};

/// Fetches over HTTPS using `tpp-network`.
///
/// # Why downloads arrive in chunks
///
/// `HttpClient` answers with a `Vec<u8>` — the whole body, in memory, with no
/// streaming API and no progress signal. Reading a 358 MB model that way would
/// hold all of it in RAM and show nothing to the person waiting.
///
/// So a download is a sequence of ranged requests instead. That costs a request
/// per chunk and buys three things the shape has to have anyway: memory stays
/// bounded, progress is real rather than a spinner, and an interrupted transfer
/// resumes at a chunk boundary (invariant 10) rather than starting again.
pub struct HttpFetcher {
    /// Its own runtime, because the client is async and this port is not.
    ///
    /// Owned rather than borrowed from a caller: `Runtime::block_on` panics if
    /// it runs on a runtime worker thread, so taking a `Handle` would make the
    /// failure depend on where a caller happened to call from. A download is
    /// blocking IO and belongs on a blocking pool regardless (invariant 5).
    runtime: tokio::runtime::Runtime,
    /// The connected client. Each request supplies its own URL, so this holds
    /// the timeout and retry policy rather than a destination.
    client: HttpClient,
}

/// Bytes per ranged request.
///
/// Large enough that the per-request overhead is noise against the transfer,
/// small enough that an interruption loses little and progress moves visibly.
const CHUNK: u64 = 8 * 1024 * 1024;

/// How many times a single chunk is retried before the download fails.
const RETRIES: usize = 3;

/// The host used to satisfy connect-time URL validation. See [`HttpFetcher::new`].
const HOST: &str = "huggingface.co";

impl HttpFetcher {
    /// Build a fetcher with its own runtime.
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Fetch {
                url: String::new(),
                reason: format!("could not start a runtime for downloads: {e}"),
            })?;

        let mut client = HttpClient::default();
        let options = ConnectionOptions {
            // A placeholder destination, and it has to be a real one: `connect`
            // builds and validates this URL even when the reachability probe is
            // skipped, so the default's empty host fails before any request is
            // made. Every actual request passes its own `UrlOptions`, so this
            // decides nothing beyond passing that validation.
            url: UrlOptions {
                scheme: UrlScheme::Https,
                host: HOST.to_owned(),
                paths: vec!["/".to_owned()],
                ..UrlOptions::default()
            },
            // No preflight: the probe would test the placeholder above rather
            // than anything this fetcher goes on to read.
            skip_connectivity_check: true,
            retries: RETRIES,
            ..ConnectionOptions::default()
        };
        runtime
            .block_on(client.connect(options))
            .map_err(|e| Error::Fetch {
                url: String::new(),
                reason: e.to_string(),
            })?;

        Ok(Self { runtime, client })
    }

    /// One GET, with the given headers.
    fn get(&self, url: &str, headers: HashMap<String, String>) -> Result<Vec<u8>> {
        let options = net_url::split(url)?;
        self.runtime
            .block_on(self.client.request_sync(
                HttpMethod::Get,
                &options,
                Vec::new(),
                &headers,
                0,
                RETRIES,
                None,
            ))
            .map_err(|e| Error::Fetch {
                url: url.to_owned(),
                reason: e.to_string(),
            })
    }
}

/// The `Range` header for a half-open byte range.
fn range_header(offset: u64, len: u64) -> HashMap<String, String> {
    let last = offset.saturating_add(len).saturating_sub(1);
    HashMap::from([("Range".to_owned(), format!("bytes={offset}-{last}"))])
}

impl Fetcher for HttpFetcher {
    fn range(&self, url: &str, offset: u64, len: u64) -> Result<Vec<u8>> {
        let mut bytes = self.get(url, range_header(offset, len))?;
        // A host that ignores the range answers with the whole file. Truncating
        // keeps the contract — the caller asked for a prefix and gets one — and
        // this is the normal path for a header peek against a plain file server.
        bytes.truncate(usize::try_from(len).unwrap_or(usize::MAX));
        Ok(bytes)
    }

    fn text(&self, url: &str) -> Result<String> {
        let bytes = self.get(url, HashMap::new())?;
        String::from_utf8(bytes).map_err(|e| Error::Fetch {
            url: url.to_owned(),
            reason: format!("the answer was not text: {e}"),
        })
    }

    fn stream(
        &self,
        url: &str,
        offset: u64,
        sink: &mut dyn std::io::Write,
        progress: &mut dyn FnMut(u64) -> bool,
    ) -> Result<()> {
        let mut at = offset;
        loop {
            let chunk = self.get(url, range_header(at, CHUNK))?;
            if chunk.is_empty() {
                // A range past the end answers empty, which is how the end of
                // the file is detected without having been told its length.
                return Ok(());
            }
            sink.write_all(&chunk)?;
            at += chunk.len() as u64;
            // A chunk boundary is the only place a cancel can be honoured, and
            // it is enough: the partial file is kept, so resuming continues
            // from here rather than from zero.
            if !progress(at) {
                return Ok(());
            }

            // A short chunk means the range ran past the end, so this was the
            // last one. Asking again would cost a round trip to learn nothing.
            if (chunk.len() as u64) < CHUNK {
                return Ok(());
            }
        }
    }
}
