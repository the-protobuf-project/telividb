//! Text in, ranked text out — with no model on the client.
//!
//! The end-to-end claim of the whole SDK: a caller sends a sentence, the
//! server embeds it, and a later question retrieves it by meaning. Every layer
//! has to agree for this to pass — the proto's `text` fields, the server's
//! inference wiring, the tokenizer, the encoder, storage and search.
//!
//! **Skipped when no model is present.** The GGUF is 80 MiB and not committed;
//! `examples/models/download.sh` fetches it. Skipping rather than failing so a
//! fresh clone's test run is green, and reporting the skip loudly so it cannot
//! be mistaken for a pass.

mod common;

use common::model_server;

/// The corpus, spanning topics far enough apart that a wrong ranking is
/// unmistakable rather than a judgement call.
const DOCUMENTS: &[(&str, &str)] = &[
    ("cat", "The cat sat quietly on the woven mat by the window."),
    (
        "rust",
        "Rust guarantees memory safety without a garbage collector.",
    ),
    (
        "ann",
        "Approximate nearest neighbour indexes trade recall for latency.",
    ),
    (
        "rain",
        "Heavy rain is expected across the region through the weekend.",
    ),
];

#[tokio::test]
async fn text_is_embedded_by_the_server_and_found_by_meaning() {
    let Some((client, _dir)) = model_server().await else {
        eprintln!("SKIPPED: no GGUF model; run examples/models/download.sh");
        return;
    };
    let mut docs = client.collection("semantic");

    let entries: Vec<(String, String)> = DOCUMENTS
        .iter()
        .map(|(id, text)| ((*id).to_owned(), (*text).to_owned()))
        .collect();
    docs.add_texts("text", &entries).await.expect("add texts");

    // Each question belongs to exactly one document. A pipeline that is subtly
    // wrong — dropped task prefix, misread pooling — still returns well-formed
    // vectors, and shows up here as the wrong document ranked first.
    for (question, expected) in [
        ("Where did the cat sit?", "cat"),
        ("How does Rust prevent memory bugs?", "rust"),
        ("Will it rain this weekend?", "rain"),
        ("What makes similarity search fast?", "ann"),
    ] {
        let found = docs.search_text("text", question, 4).await.expect("search");
        assert_eq!(
            found.hits()[0].name,
            expected,
            "{question:?} ranked {:?} first",
            found.hits()
        );
    }
}

#[tokio::test]
async fn a_hit_carries_the_text_it_was_stored_with() {
    let Some((client, _dir)) = model_server().await else {
        eprintln!("SKIPPED: no GGUF model; run examples/models/download.sh");
        return;
    };
    let mut docs = client.collection("withtext");

    docs.add_text("cat", "text", DOCUMENTS[0].1)
        .await
        .expect("add");

    let found = docs
        .search_text("text", "where did the cat sit", 1)
        .await
        .expect("search");
    assert_eq!(found.hits()[0].text.as_deref(), Some(DOCUMENTS[0].1));
}

#[tokio::test]
async fn a_server_without_a_model_refuses_text_instead_of_storing_nothing() {
    // The failure that would otherwise be silent: accepting the text, storing
    // no vector, and reporting success. The message has to name the flag.
    let (client, _dir) = common::connected().await;
    let mut docs = client.collection("nomodel");

    match docs.add_text("doc-1", "text", "anything").await {
        Err(telividb_client::Error::Server { message, .. }) => {
            assert!(message.contains("--model"), "got {message}");
        }
        other => panic!("expected a refusal naming --model, got {other:?}"),
    }
}
