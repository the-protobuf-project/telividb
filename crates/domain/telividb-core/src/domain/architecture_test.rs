use super::Architecture;

#[test]
fn every_name_round_trips_through_recognition() {
    // `NAMES` and `from_gguf` are two statements of the same set, and a catalog
    // gate reads the first while a loader reads the second. If they disagree,
    // a model passes the download check and fails at load.
    for name in Architecture::NAMES {
        let arch = Architecture::from_gguf(name).expect("a listed name is recognised");
        assert_eq!(arch.as_str(), *name);
    }
}

#[test]
fn architectures_this_engine_cannot_read_are_refused() {
    // Each of these is a real embedding model published as GGUF, and none of
    // them loads here. They are named rather than represented by a nonsense
    // string because the point is that *plausible* input is refused: the first
    // three are encoders whose output would look entirely reasonable.
    for unsupported in [
        "gemma-embedding", // EmbeddingGemma
        "qwen3",           // Qwen3-Embedding
        "llama",           // E5-Mistral, and every generative model
        "clip",            // image towers
        "whisper",         // audio
    ] {
        assert_eq!(Architecture::from_gguf(unsupported), None, "{unsupported}");
    }
}
