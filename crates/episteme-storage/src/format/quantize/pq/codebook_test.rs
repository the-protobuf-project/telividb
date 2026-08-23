use super::*;

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
fn encodes_to_one_byte_per_subspace() {
    let (book, data) = fit(64, 8);
    assert_eq!(book.encode(&data[0]).unwrap().len(), 8);
}

#[test]
fn compression_beats_int8_at_the_same_width() {
    // The reason PQ exists: a byte per subspace rather than per dimension.
    // At d=768 with m=96, a row is 96 bytes against int8's 776 and f32's 3072.
    let (book, _) = fit(768, 96);
    let pq_bytes = book.m();
    let int8_bytes = 768 + 8;
    let f32_bytes = 768 * 4;

    assert_eq!(pq_bytes, 96);
    assert!(pq_bytes * 8 < int8_bytes, "~8x better than int8");
    assert_eq!(f32_bytes / pq_bytes, 32, "32x against full precision");
}

#[test]
fn reconstruction_is_close_for_clustered_data() {
    let (book, data) = fit(64, 8);
    let mut worst = 0f32;
    for v in data.iter().take(50) {
        let back = book.decode(&book.encode(v).unwrap()).unwrap();
        let error = kmeans::l2(v, &back).sqrt() / (v.len() as f32).sqrt();
        worst = worst.max(error);
    }
    assert!(worst < 0.5, "per-component error {worst} is too large");
}

#[test]
fn more_subspaces_reconstruct_more_accurately() {
    // The central trade-off, asserted rather than assumed: more subspaces means
    // more bytes and less error.
    let data = training(600, 64, 9);
    let refs = as_refs(&data);

    let error_at = |m: usize| {
        let book = PqCodebook::train(
            &refs,
            64,
            PqParams {
                m,
                ..Default::default()
            },
        )
        .unwrap();
        data.iter()
            .take(50)
            .map(|v| kmeans::l2(v, &book.decode(&book.encode(v).unwrap()).unwrap()))
            .sum::<f32>()
    };

    assert!(
        error_at(16) < error_at(4),
        "more subspaces should reconstruct better"
    );
}

#[test]
fn ranking_survives_for_well_separated_vectors() {
    let (book, data) = fit(64, 8);
    let query = &data[0];
    let near = &data[16]; // same cluster: 16 centres, stride 16
    let far = &data[8];

    let dot = |a: &[f32], b: &[f32]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
    let near_q = book.decode(&book.encode(near).unwrap()).unwrap();
    let far_q = book.decode(&book.encode(far).unwrap()).unwrap();

    let exact_prefers_near = dot(query, near) > dot(query, far);
    let quantized_prefers_near = dot(query, &near_q) > dot(query, &far_q);
    assert_eq!(
        exact_prefers_near, quantized_prefers_near,
        "quantization flipped the ordering"
    );
}

#[test]
fn a_width_that_does_not_divide_evenly_is_refused() {
    // Padding silently would make the last subspace partly meaningless.
    let data = training(100, 10, 1);
    let err = PqCodebook::train(
        &as_refs(&data),
        10,
        PqParams {
            m: 3,
            ..Default::default()
        },
    );
    assert!(matches!(err, Err(Error::InvalidPqShape { .. })));
}

#[test]
fn zero_subspaces_are_refused() {
    let data = training(100, 8, 1);
    assert!(
        PqCodebook::train(
            &as_refs(&data),
            8,
            PqParams {
                m: 0,
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn encoding_a_wrong_width_vector_is_refused() {
    let (book, _) = fit(64, 8);
    assert!(matches!(
        book.encode(&[1.0; 32]),
        Err(Error::PqDimMismatch { .. })
    ));
}

#[test]
fn decoding_a_wrong_length_code_run_is_refused() {
    let (book, _) = fit(64, 8);
    assert!(matches!(
        book.decode(&[0u8; 3]),
        Err(Error::PqDimMismatch { .. })
    ));
}

#[test]
fn training_is_deterministic() {
    let data = training(300, 32, 4);
    let refs = as_refs(&data);
    let a = PqCodebook::train(&refs, 32, PqParams::default()).unwrap();
    let b = PqCodebook::train(&refs, 32, PqParams::default()).unwrap();
    assert_eq!(
        a, b,
        "a codebook is baked into every code written against it"
    );
}
