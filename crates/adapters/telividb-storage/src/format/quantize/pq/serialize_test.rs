use super::*;
use crate::error::Error;
use crate::format::quantize::PqParams;
use telividb_distance::cluster as kmeans;
use telividb_distance::pq::PqCodebook;

/// Clustered training data — the shape PQ is designed for. Uniform noise has no
/// structure for a codebook to capture, so it would understate the codec badly.
fn training(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = kmeans::Rng(seed);
    let centres: Vec<Vec<f32>> = (0..16)
        .map(|_| {
            (0..dim)
                .map(|_| (rng.next_u64() % 2000) as f32 / 1000.0 - 1.0)
                .collect()
        })
        .collect();

    (0..count)
        .map(|i| {
            centres[i % centres.len()]
                .iter()
                .map(|c| c + ((rng.next_u64() % 200) as f32 / 1000.0 - 0.1))
                .collect()
        })
        .collect()
}

fn as_refs(v: &[Vec<f32>]) -> Vec<&[f32]> {
    v.iter().map(Vec::as_slice).collect()
}

fn fit(dim: usize, m: usize) -> (PqCodebook, Vec<Vec<f32>>) {
    let data = training(600, dim, 5);
    let book = PqCodebook::train(
        &as_refs(&data),
        dim,
        PqParams {
            m,
            ..Default::default()
        },
    )
    .expect("valid shape");
    (book, data)
}

#[test]
fn serialization_round_trips() {
    let (book, data) = fit(32, 4);
    let mut bytes = Vec::new();
    encode_codebook(&book, &mut bytes);

    assert_eq!(bytes.len(), encoded_len(&book));
    let back = decode_codebook(&bytes).unwrap();
    assert_eq!(back, book);
    assert_eq!(
        back.encode(&data[0]).unwrap(),
        book.encode(&data[0]).unwrap()
    );
}

#[test]
fn a_foreign_codebook_is_rejected() {
    let (book, _) = fit(32, 4);
    let mut bytes = Vec::new();
    encode_codebook(&book, &mut bytes);
    bytes[0..4].copy_from_slice(b"XXXX");
    assert!(matches!(
        decode_codebook(&bytes),
        Err(Error::BadMagic { .. })
    ));
}

#[test]
fn a_newer_version_is_refused() {
    let (book, _) = fit(32, 4);
    let mut bytes = Vec::new();
    encode_codebook(&book, &mut bytes);
    bytes[4..6].copy_from_slice(&(CODEBOOK_VERSION + 1).to_le_bytes());
    assert!(matches!(
        decode_codebook(&bytes),
        Err(Error::UnsupportedVersion { .. })
    ));
}

#[test]
fn a_truncated_codebook_is_an_error_never_an_overrun() {
    // Codebooks arrive inside archives, so this is untrusted input.
    let (book, _) = fit(32, 4);
    let mut bytes = Vec::new();
    encode_codebook(&book, &mut bytes);
    for cut in [0usize, 8, 13, 20, bytes.len() - 1] {
        assert!(decode_codebook(&bytes[..cut]).is_err(), "cut {cut}");
    }
}

#[test]
fn a_lying_dimension_header_does_not_overrun() {
    let (book, _) = fit(32, 4);
    let mut bytes = Vec::new();
    encode_codebook(&book, &mut bytes);
    bytes[6..10].copy_from_slice(&65536u32.to_le_bytes());
    assert!(decode_codebook(&bytes).is_err());
}
