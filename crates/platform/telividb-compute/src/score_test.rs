use super::*;
use crate::backend::Backend;
use crate::device::DeviceKind;

/// A deterministic corpus, distinct enough that a wrong answer is obvious.
fn corpus_data(rows: usize, dim: usize) -> Vec<f32> {
    (0..rows * dim)
        .map(|i| ((i % 17) as f32) * 0.1 - 0.8)
        .collect()
}

/// The inner product, computed on the host — the reference every device
/// result is checked against.
fn expected(data: &[f32], queries: &[f32], rows: usize, dim: usize, count: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(count * rows);
    for q in 0..count {
        let query = &queries[q * dim..(q + 1) * dim];
        for r in 0..rows {
            let row = &data[r * dim..(r + 1) * dim];
            out.push(row.iter().zip(query).map(|(a, b)| a * b).sum::<f32>());
        }
    }
    out
}

fn upload(rows: usize, dim: usize) -> (Corpus, Vec<f32>) {
    let data = corpus_data(rows, dim);
    let backend = Backend::best().expect("a backend is always available");
    let corpus = Corpus::upload(backend, &data, rows, dim).expect("upload");
    (corpus, data)
}

#[test]
fn scores_match_the_host_computation() {
    // The claim the whole crate rests on: a device result is the same result,
    // only faster. A silent divergence here would rank everything subtly wrong
    // while erroring nowhere.
    const ROWS: usize = 64;
    const DIM: usize = 16;
    const COUNT: usize = 5;

    let (corpus, data) = upload(ROWS, DIM);
    let queries: Vec<f32> = (0..COUNT * DIM)
        .map(|i| ((i % 7) as f32) * 0.2 - 0.5)
        .collect();

    let scores = corpus.score_batch(&queries, COUNT).expect("score");
    let want = expected(&data, &queries, ROWS, DIM, COUNT);

    assert_eq!(scores.queries(), COUNT);
    assert_eq!(scores.rows(), ROWS);
    for (got, expect) in scores.as_slice().iter().zip(&want) {
        assert!(
            (got - expect).abs() < 1e-4,
            "device scored {got}, host scored {expect}"
        );
    }
}

#[test]
fn each_query_gets_its_own_row_of_scores() {
    // A flat buffer of `count * rows` is trivially indexed wrongly, which is
    // why `Scores` carries the shape. This pins that `row(i)` really is
    // query `i` and not a slice straddling two.
    const ROWS: usize = 32;
    const DIM: usize = 8;

    let (corpus, data) = upload(ROWS, DIM);
    let queries: Vec<f32> = (0..3 * DIM).map(|i| (i as f32) * 0.05).collect();
    let scores = corpus.score_batch(&queries, 3).expect("score");

    for q in 0..3 {
        let row = scores.row(q).expect("every query has a row");
        let single = expected(&data, &queries[q * DIM..(q + 1) * DIM], ROWS, DIM, 1);
        for (got, expect) in row.iter().zip(&single) {
            assert!((got - expect).abs() < 1e-4, "query {q}: {got} vs {expect}");
        }
    }
    assert!(scores.row(3).is_none(), "there is no fourth query");
}

#[test]
fn a_batch_matches_scoring_one_at_a_time() {
    // Batching may change how the work is scheduled, never what it produces.
    const ROWS: usize = 48;
    const DIM: usize = 12;

    let (corpus, _) = upload(ROWS, DIM);
    let queries: Vec<f32> = (0..4 * DIM).map(|i| ((i % 11) as f32) * 0.1).collect();

    let batched = corpus.score_batch(&queries, 4).expect("batched");
    for q in 0..4 {
        let single = corpus
            .score_batch(&queries[q * DIM..(q + 1) * DIM], 1)
            .expect("single");
        for (a, b) in batched.row(q).unwrap().iter().zip(single.as_slice()) {
            assert!((a - b).abs() < 1e-5, "query {q}: batched {a}, single {b}");
        }
    }
}

#[test]
fn the_host_backend_agrees_with_the_default_one() {
    // On a machine with a GPU these are different backends. Identical results
    // are what makes the CPU fallback safe rather than merely available.
    const ROWS: usize = 40;
    const DIM: usize = 8;

    let data = corpus_data(ROWS, DIM);
    let queries: Vec<f32> = (0..2 * DIM).map(|i| (i as f32) * 0.07 - 0.3).collect();

    let host = Corpus::upload(Backend::of(DeviceKind::Cpu).expect("cpu"), &data, ROWS, DIM)
        .expect("host upload");
    let best = Corpus::upload(Backend::best().expect("best"), &data, ROWS, DIM).expect("upload");

    let a = host.score_batch(&queries, 2).expect("host");
    let b = best.score_batch(&queries, 2).expect("best");
    for (x, y) in a.as_slice().iter().zip(b.as_slice()) {
        assert!(
            (x - y).abs() < 1e-4,
            "host {x} vs {} {y}",
            best.backend().device().kind().as_str()
        );
    }
}

#[test]
fn an_empty_batch_scores_nothing_rather_than_erroring() {
    let (corpus, _) = upload(16, 4);
    let scores = corpus.score_batch(&[], 0).expect("empty batch");
    assert_eq!(scores.queries(), 0);
    assert!(scores.as_slice().is_empty());
}

#[test]
fn a_query_buffer_of_the_wrong_length_is_refused() {
    // Refused rather than truncated: a short buffer would score against
    // whatever happened to follow it in memory.
    let (corpus, _) = upload(16, 4);
    assert!(corpus.score_batch(&[1.0, 2.0], 1).is_err());
}

#[test]
fn an_empty_corpus_is_refused_at_upload() {
    let backend = Backend::best().expect("backend");
    assert!(Corpus::upload(backend, &[], 0, 8).is_err());
}

#[test]
fn a_corpus_reports_what_it_holds() {
    let (corpus, _) = upload(100, 32);
    assert_eq!(corpus.rows(), 100);
    assert_eq!(corpus.dim(), 32);
    assert_eq!(corpus.bytes(), 100 * 32 * 4);
}
