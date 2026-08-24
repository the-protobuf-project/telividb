//! What `PqCodebook::train` refuses.
//!
//! Split from `codebook_test.rs` because it asks a different question: not
//! whether a trained codebook encodes correctly, but which training sets
//! cannot produce one at all.

use super::{PqCodebook, PqParams};
use crate::error::Error;

#[test]
fn training_on_nothing_is_refused() {
    // With no training vectors, seeding returns zeros and the update loop
    // breaks before it runs — so this used to return `Ok` with a codebook in
    // which every row encodes to code 0. The tier then ranks nothing, silently,
    // and recall is zero with no error anywhere.
    let error = PqCodebook::train(
        &[],
        16,
        PqParams {
            m: 4,
            ..Default::default()
        },
    )
    .expect_err("an empty training set cannot produce a codebook");
    assert!(
        matches!(error, Error::PqTrainingTooSmall { found: 0, .. }),
        "unexpected error: {error}"
    );
}

#[test]
fn training_below_the_centroid_count_is_refused() {
    // Fewer vectors than centroids cannot fill the codebook either. It degrades
    // quietly rather than failing, which is the worse direction.
    let rows: Vec<Vec<f32>> = (0..100).map(|i| vec![i as f32; 16]).collect();
    let refs: Vec<&[f32]> = rows.iter().map(Vec::as_slice).collect();

    let error = PqCodebook::train(
        &refs,
        16,
        PqParams {
            m: 4,
            ..Default::default()
        },
    )
    .expect_err("100 vectors cannot fill 256 centroids");
    assert!(matches!(
        error,
        Error::PqTrainingTooSmall { found: 100, .. }
    ));
}

#[test]
fn training_at_the_centroid_count_succeeds() {
    // The boundary is inclusive: exactly one vector per centroid is the least
    // that can still distinguish rows.
    let rows: Vec<Vec<f32>> = (0..256).map(|i| vec![i as f32; 16]).collect();
    let refs: Vec<&[f32]> = rows.iter().map(Vec::as_slice).collect();
    assert!(
        PqCodebook::train(
            &refs,
            16,
            PqParams {
                m: 4,
                ..Default::default()
            }
        )
        .is_ok()
    );
}
