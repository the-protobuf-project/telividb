//! The several ways a person names a model.

/// What a pasted string refers to.
///
/// People arrive with whatever they had in the clipboard — the address bar of
/// a model page, a direct download link, or the `owner/name` from a README.
/// All three mean the same thing, and refusing two of them because they are
/// not the canonical form is the kind of friction the catalog exists to remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    /// A repository, which may hold several quantizations to choose between.
    Repository {
        /// Which repository, as `owner/name`; the file is still to be chosen.
        repo: String,
    },
    /// One specific file in a repository.
    File {
        /// Which repository the file lives in, as `owner/name`.
        repo: String,
        /// Which revision — a branch, a tag or a commit.
        ///
        /// Kept rather than dropped. An earlier version normalised every link
        /// to `main` on the grounds that the catalog pins it; that reasoning
        /// covered catalog entries and not pasted links, where discarding the
        /// revision silently fetches a *different file* than the one the person
        /// asked for.
        revision: String,
        /// Which file, chosen already — so no quantization prompt is needed.
        file: String,
    },
    /// A URL somewhere other than the model host.
    ///
    /// Fetched and inspected like anything else — the architecture gate and
    /// the header check do not care where bytes come from — but with no API to
    /// ask, so its size and digest are only known once it has been read.
    Url {
        /// The address, verbatim.
        url: String,
    },
}

/// The model host this crate resolves repositories against.
const HOST: &str = "huggingface.co";

impl Reference {
    /// Interpret a string a person pasted.
    ///
    /// Returns `None` only for input with no plausible reading at all — an
    /// empty string, or a bare word that is neither a URL nor `owner/name`.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }
        if let Some(rest) = strip_host(input) {
            return Some(Self::from_host_path(rest));
        }
        if input.starts_with("http://") || input.starts_with("https://") {
            return Some(Self::Url {
                url: input.to_owned(),
            });
        }
        // `owner/name`, and nothing deeper: a third segment is a path into a
        // repository, which without a host is too ambiguous to guess at.
        let mut parts = input.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(owner), Some(name), None) if !owner.is_empty() && !name.is_empty() => {
                Some(Self::Repository {
                    repo: format!("{owner}/{name}"),
                })
            }
            _ => None,
        }
    }

    /// Read the path part of a model-host URL.
    ///
    /// Handles the two shapes that actually appear: a repository page, and a
    /// `resolve` link to one file. Anything else in the repository — a blob
    /// view, a commit, a discussion — resolves to the repository itself, which
    /// is the useful answer rather than an error.
    fn from_host_path(path: &str) -> Self {
        let path = path.split(['?', '#']).next().unwrap_or(path);
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let [owner, name, rest @ ..] = segments.as_slice() else {
            return Self::Url {
                url: format!("https://{HOST}/{path}"),
            };
        };
        let repo = format!("{owner}/{name}");
        // `.../resolve/<ref>/<file>` and `.../blob/<ref>/<file>` both name one
        // file at one revision, and the revision is carried through: a link to a
        // tag or a commit means that revision, not whatever `main` holds today.
        match rest {
            ["resolve" | "blob", reference, file @ ..] if !file.is_empty() => Self::File {
                repo,
                revision: (*reference).to_owned(),
                file: file.join("/"),
            },
            _ => Self::Repository { repo },
        }
    }

    /// The repository this refers to, if any.
    pub fn repository(&self) -> Option<&str> {
        match self {
            Self::Repository { repo } | Self::File { repo, .. } => Some(repo),
            Self::Url { .. } => None,
        }
    }
}

/// Strip a model-host prefix, with or without a scheme or `www.`.
fn strip_host(input: &str) -> Option<&str> {
    let rest = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .unwrap_or(input);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    rest.strip_prefix(HOST)?.strip_prefix('/')
}

#[cfg(test)]
#[path = "reference_test.rs"]
mod tests;
