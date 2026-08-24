//! Which side of a search a text is on.

/// Whether text is being stored or used to search.
///
/// This exists because asymmetric embedding models encode the two differently.
/// nomic-embed and e5 prepend a task prefix (`search_document: ` versus
/// `search_query: `), and the model was trained with it — dropping the prefix,
/// or using the wrong one, measurably lowers recall while returning
/// well-formed vectors that look entirely normal.
///
/// It is the same distinction invariant 18 draws for a joint model, where a
/// query against an image field must route to the text tower. Making it an
/// argument at the boundary means the caller has to have decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    /// Text being embedded for storage.
    Document,
    /// Text being embedded to search with.
    Query,
}

impl Task {
    /// The name used in configuration and telemetry.
    pub fn as_str(self) -> &'static str {
        match self {
            Task::Document => "document",
            Task::Query => "query",
        }
    }
}
