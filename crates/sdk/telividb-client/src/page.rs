//! Following a paginated listing to its end.
//!
//! Shared by every `list_*` in this SDK because the shape is the same each time:
//! ask for a page, keep the rows, follow the token until it is empty. A caller
//! asking for "the projects" wants all of them, and a page token is an artefact
//! of the transport rather than something a window should carry.

use crate::error::Result;

/// How many to ask for per page. Large enough that one round trip usually
/// suffices, small enough that a huge tenant does not arrive in one message.
pub(crate) const PAGE_SIZE: i32 = 200;

/// Refuse a page token the server has already handed out.
///
/// A server that returns the same token twice describes a cycle, and following
/// it is an unbounded loop that accumulates rows until the process dies — a hang
/// with no error, which is the worst shape a bug can take in a client library.
/// Detected rather than trusted, because the pagination contract is the server's
/// to honour and this side cannot make it.
pub(crate) fn advance(seen: &mut Vec<String>, token: String) -> Result<String> {
    if seen.iter().any(|t| t == &token) {
        return Err(crate::Error::InvalidArgument {
            message: format!(
                "the server repeated page token {token:?}, which would loop \
                 forever; its pagination is not advancing"
            ),
        });
    }
    seen.push(token.clone());
    Ok(token)
}

