//! A small text corpus, so the examples have something real to search.

/// Sentences spanning four unrelated topics.
///
/// Deliberately grouped: within-topic pairs should score above across-topic
/// ones, which is what makes a wrong result *visible* in the output rather
/// than merely a number the reader has no way to judge.
pub const DOCUMENTS: &[&str] = &[
    // Animals.
    "The cat sat quietly on the woven mat by the window.",
    "A feline rested upon the rug, watching the birds outside.",
    "Dogs greet their owners enthusiastically after a long day.",
    // Systems programming.
    "Rust guarantees memory safety without a garbage collector.",
    "Ownership and borrowing let the compiler prove lifetimes correct.",
    "Zero-cost abstractions compile down to the same code you would write by hand.",
    // Databases.
    "A vector database stores embeddings and searches them by similarity.",
    "Approximate nearest neighbour indexes trade recall for query latency.",
    "Sealed segments are immutable, which is what makes lock-free reads safe.",
    // Weather.
    "Heavy rain is expected across the region through the weekend.",
    "The forecast calls for clear skies and unusually warm temperatures.",
    "A cold front will move in overnight, bringing a sharp drop in temperature.",
];

/// Queries chosen to have an obviously right answer.
///
/// If the pipeline is wired correctly these retrieve their own topic. If
/// something is subtly wrong — the task prefix dropped, pooling misread, a
/// rotation convention mismatched — the vectors stay well-formed and these
/// rankings go visibly wrong, which is the point of picking them this way.
pub const QUERIES: &[&str] = &[
    "Where did the cat sit?",
    "How does Rust prevent memory bugs?",
    "What makes similarity search fast?",
    "Will it rain this weekend?",
];
