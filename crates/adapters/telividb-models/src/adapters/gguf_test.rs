use super::GgufHeader;

/// Build a GGUF header with the given metadata pairs.
///
/// Synthetic rather than a committed fixture: a real model file is hundreds of
/// megabytes, and every case worth testing here — a truncated prefix, a
/// missing architecture, a huge vocabulary array to skip — is easier to state
/// as bytes than to find in the wild.
fn gguf(pairs: &[(&str, Value)]) -> Vec<u8> {
    let mut out = b"GGUF".to_vec();
    out.extend(3u32.to_le_bytes()); // format version
    out.extend(0u64.to_le_bytes()); // tensor count
    out.extend((pairs.len() as u64).to_le_bytes());
    for (key, value) in pairs {
        out.extend((key.len() as u64).to_le_bytes());
        out.extend(key.as_bytes());
        value.write(&mut out);
    }
    out
}

/// The metadata value kinds these tests need.
enum Value {
    Str(&'static str),
    U32(u32),
    /// A counted array of strings, standing in for a tokenizer vocabulary.
    Strings(usize),
}

impl Value {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Str(s) => {
                out.extend(8u32.to_le_bytes());
                out.extend((s.len() as u64).to_le_bytes());
                out.extend(s.as_bytes());
            }
            Self::U32(v) => {
                out.extend(4u32.to_le_bytes());
                out.extend(v.to_le_bytes());
            }
            Self::Strings(n) => {
                out.extend(9u32.to_le_bytes()); // array
                out.extend(8u32.to_le_bytes()); // of strings
                out.extend((*n as u64).to_le_bytes());
                for i in 0..*n {
                    let tok = format!("tok{i}");
                    out.extend((tok.len() as u64).to_le_bytes());
                    out.extend(tok.as_bytes());
                }
            }
        }
    }
}

#[test]
fn reads_architecture_and_shape_past_a_vocabulary() {
    // The vocabulary sits between the fields that matter in a real file, and
    // it is the one array that cannot be skipped arithmetically. If skipping
    // it were wrong, the fields after it would read as garbage rather than
    // fail — so this is the case worth pinning.
    let bytes = gguf(&[
        ("general.architecture", Value::Str("bert")),
        ("tokenizer.ggml.tokens", Value::Strings(30_000)),
        ("bert.embedding_length", Value::U32(384)),
        ("bert.context_length", Value::U32(512)),
    ]);
    let header = GgufHeader::parse(&bytes).expect("a well-formed header");
    assert_eq!(header.architecture, "bert");
    assert_eq!(header.dimensions, Some(384));
    assert_eq!(header.context_length, Some(512));
    assert!(header.supported().is_some());
}

#[test]
fn a_truncated_prefix_keeps_what_it_read() {
    // The header is fetched as a range request, so ending mid-file is the
    // normal case rather than corruption. Everything read before the cut is
    // still usable, and the architecture is what the gate needs.
    let bytes = gguf(&[
        ("general.architecture", Value::Str("nomic-bert")),
        ("nomic-bert.embedding_length", Value::U32(768)),
        ("tokenizer.ggml.tokens", Value::Strings(30_000)),
    ]);
    let header = GgufHeader::parse(&bytes[..600]).expect("a truncated but usable prefix");
    assert_eq!(header.architecture, "nomic-bert");
    assert_eq!(header.dimensions, Some(768));
}

#[test]
fn an_html_error_page_is_reported_as_such() {
    // A model host answers a bad path with an HTML page and status 200. Read
    // as a model this is simply "not GGUF", and saying so is what stops the
    // failure surfacing later as a corrupt-file report.
    let err = GgufHeader::parse(b"<!DOCTYPE html><html>404</html>")
        .unwrap_err()
        .to_string();
    assert!(err.contains("GGUF"), "{err}");
}

#[test]
fn a_header_without_an_architecture_is_refused() {
    let bytes = gguf(&[("general.name", Value::Str("mystery"))]);
    let err = GgufHeader::parse(&bytes).unwrap_err().to_string();
    assert!(err.contains("general.architecture"), "{err}");
}

#[test]
fn an_unsupported_architecture_names_itself_and_the_alternatives() {
    let bytes = gguf(&[
        ("general.architecture", Value::Str("gemma-embedding")),
        ("gemma-embedding.embedding_length", Value::U32(768)),
    ]);
    let header = GgufHeader::parse(&bytes).expect("parses fine; it just cannot be loaded");
    assert!(header.supported().is_none());

    let err = header
        .require_supported("ggml-org/embeddinggemma-300m")
        .unwrap_err()
        .to_string();
    assert!(err.contains("gemma-embedding"), "{err}");
    assert!(err.contains("bert"), "{err}");
    assert!(err.contains("ggml-org/embeddinggemma-300m"), "{err}");
}
