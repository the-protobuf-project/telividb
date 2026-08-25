use super::*;
use crate::adapters::MemoryStore;
use crate::adapters::gpu::load::{load_corpus, metric_of};
use telividb_core::VectorStore;

const DIM: u32 = 4;

fn dim() -> Dim {
    Dim::new(DIM).unwrap()
}

/// Rows 0 and 2 present, row 1 absent — the multimodal shape invariant 17
/// describes, where a point simply has no value for this field.
fn store_with_a_gap() -> MemoryStore {
    let mut store = MemoryStore::new(dim(), Metric::Dot);
    store.push(&[1.0, 2.0, 3.0, 4.0]).unwrap();
    store.push_absent();
    store.push(&[5.0, 6.0, 7.0, 8.0]).unwrap();
    store
}

fn round_trip(store: &dyn VectorStore) -> Corpus {
    let mut buffer = std::io::Cursor::new(Vec::new());
    write_corpus(store, &mut buffer).unwrap();
    buffer.set_position(0);
    load_corpus(&mut buffer, &Device::Cpu).unwrap()
}

#[test]
fn vectors_survive_the_round_trip_bit_exactly() {
    // Not "close enough": the GPU index's correctness test is equality with
    // the CPU flat index, which only means something if the corpus itself is
    // unchanged. F32 storage is chosen for exactly this.
    let store = store_with_a_gap();
    let corpus = round_trip(&store);

    let flat: Vec<f32> = corpus
        .tensor
        .flatten_all()
        .unwrap()
        .to_vec1::<f32>()
        .unwrap();
    assert_eq!(&flat[0..4], &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(&flat[8..12], &[5.0, 6.0, 7.0, 8.0]);
}

#[test]
fn an_absent_row_keeps_its_slot_but_is_marked_absent() {
    // The slot must still exist, or a row's offset stops being its ordinal.
    let store = store_with_a_gap();
    let corpus = round_trip(&store);

    assert_eq!(corpus.present, vec![true, false, true]);
    let flat: Vec<f32> = corpus
        .tensor
        .flatten_all()
        .unwrap()
        .to_vec1::<f32>()
        .unwrap();
    assert_eq!(
        flat.len(),
        3 * DIM as usize,
        "absent row still occupies a slot"
    );
    assert_eq!(&flat[4..8], &[0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn dim_and_metric_survive_as_metadata() {
    let mut store = MemoryStore::new(dim(), Metric::Cosine);
    store.push(&[1.0, 0.0, 0.0, 0.0]).unwrap();
    let corpus = round_trip(&store);

    assert_eq!(corpus.dim.get(), DIM as usize);
    assert_eq!(corpus.metric, Metric::Cosine);
}

#[test]
fn every_metric_names_itself_reversibly() {
    for metric in [Metric::Dot, Metric::L2, Metric::Cosine] {
        assert_eq!(metric_of(metric_name(metric)).unwrap(), metric);
    }
}

#[test]
fn an_unknown_metric_is_refused_rather_than_guessed() {
    assert!(metric_of("manhattan").is_err());
}

#[test]
fn an_empty_store_round_trips() {
    let store = MemoryStore::new(dim(), Metric::Dot);
    let corpus = round_trip(&store);
    assert!(corpus.present.is_empty());
}
