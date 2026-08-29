//! Splitting a URL into the shape `tpp-network` wants.

use crate::{Error, Result};
use tpp_network::{UrlOptions, UrlScheme};

/// Break a URL into scheme, host, one path and its query parameters.
///
/// `tpp-network` models a URL as parts rather than a string, so this is the
/// adapter between the two. Written by hand rather than with a URL crate
/// because the inputs are narrow — a model host's API and its download links —
/// and a parser dependency for two `split` calls is not worth the weight.
pub(super) fn split(url: &str) -> Result<UrlOptions> {
    let unsupported = || Error::Fetch {
        url: url.to_owned(),
        reason: "only http and https URLs can be fetched".to_owned(),
    };

    let (scheme, rest) = match url.split_once("://") {
        Some(("https", rest)) => (UrlScheme::Https, rest),
        Some(("http", rest)) => (UrlScheme::Http, rest),
        _ => return Err(unsupported()),
    };

    let (authority, path_and_query) = match rest.split_once('/') {
        Some((host, tail)) => (host, tail),
        None => (rest, ""),
    };
    if authority.is_empty() {
        return Err(unsupported());
    }

    let (path, query) = match path_and_query.split_once('?') {
        Some((path, query)) => (path, query),
        None => (path_and_query, ""),
    };

    Ok(UrlOptions {
        scheme,
        host: authority.to_owned(),
        // One path, selected by index 0 at every call site here.
        paths: vec![format!("/{path}")],
        params: query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| match pair.split_once('=') {
                Some((k, v)) => (k.to_owned(), v.to_owned()),
                None => (pair.to_owned(), String::new()),
            })
            .collect(),
    })
}

#[cfg(test)]
#[path = "net_url_test.rs"]
mod tests;
