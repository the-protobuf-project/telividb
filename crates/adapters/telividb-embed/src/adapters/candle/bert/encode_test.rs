use super::*;
use crate::adapters::candle::fixture::{TinyModel, write_tiny_gguf};
use crate::adapters::candle::weights::Weights;
use candle_core::quantized::gguf_file::Content;

fn load() -> (tempfile::TempDir, QuantizedBert, TinyModel) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.gguf");
    let model = TinyModel::default();
    write_tiny_gguf(&path, &model).unwrap();

    let mut file = std::fs::File::open(&path).unwrap();
    let content = Content::read(&mut file).unwrap();
    let mut weights = Weights::new(content, file, Device::Cpu);
    (dir, QuantizedBert::load(&mut weights).unwrap(), model)
}

fn batch(rows: usize, seq: usize, real: usize) -> (Tensor, Tensor) {
    let ids: Vec<u32> = (0..rows * seq).map(|i| (i % 8 + 4) as u32).collect();
    let mask: Vec<u32> = (0..rows * seq)
        .map(|i| if i % seq < real { 1 } else { 0 })
        .collect();
    (
        Tensor::from_vec(ids, (rows, seq), &Device::Cpu).unwrap(),
        Tensor::from_vec(mask, (rows, seq), &Device::Cpu).unwrap(),
    )
}

#[test]
fn a_real_gguf_loads_every_tensor_the_forward_pass_needs() {
    // The whole point of a real fixture: a missing or misnamed tensor fails
    // here, where a mocked loader would agree with whatever the code does.
    let (_dir, encoder, model) = load();
    assert_eq!(encoder.hidden(), model.hidden);
    assert_eq!(encoder.context(), model.context);
    assert_eq!(encoder.architecture(), "bert");
    assert_eq!(encoder.feed_forward(), model.ff);
}

#[test]
fn the_forward_pass_produces_one_finite_vector_per_row() {
    let (_dir, encoder, model) = load();
    let (ids, mask) = batch(3, 5, 5);

    let out = encoder.forward(&ids, &mask, Pooling::Mean).unwrap();
    assert_eq!(out.dims2().unwrap(), (3, model.hidden));

    let values = out.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    assert!(values.iter().all(|v| v.is_finite()), "got {values:?}");
}

#[test]
fn padding_does_not_change_a_rows_embedding() {
    // This is the property the mask exists for. If padding leaked in, the
    // same text would embed differently depending on what else was batched
    // with it — which is invisible until recall drops.
    let (_dir, encoder, _) = load();

    let ids = Tensor::from_vec(vec![4u32, 5, 6], (1, 3), &Device::Cpu).unwrap();
    let mask = Tensor::from_vec(vec![1u32, 1, 1], (1, 3), &Device::Cpu).unwrap();
    let tight = encoder.forward(&ids, &mask, Pooling::Mean).unwrap();

    let padded_ids = Tensor::from_vec(vec![4u32, 5, 6, 0, 0], (1, 5), &Device::Cpu).unwrap();
    let padded_mask = Tensor::from_vec(vec![1u32, 1, 1, 0, 0], (1, 5), &Device::Cpu).unwrap();
    let padded = encoder
        .forward(&padded_ids, &padded_mask, Pooling::Mean)
        .unwrap();

    let a = tight.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    let b = padded.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    for (x, y) in a.iter().zip(&b) {
        assert!(
            (x - y).abs() < 1e-4,
            "padding changed the vector: {a:?} vs {b:?}"
        );
    }
}

#[test]
fn a_fully_padded_row_does_not_poison_the_batch_with_nan() {
    // An all-padding row is what a short text in a long batch produces. With
    // an `-inf` mask its softmax denominator is zero and the NaN spreads.
    let (_dir, encoder, _) = load();
    let ids = Tensor::from_vec(vec![4u32, 5, 0, 0], (2, 2), &Device::Cpu).unwrap();
    let mask = Tensor::from_vec(vec![1u32, 1, 0, 0], (2, 2), &Device::Cpu).unwrap();

    let out = encoder.forward(&ids, &mask, Pooling::Mean).unwrap();
    let values = out.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    assert!(values.iter().all(|v| v.is_finite()), "got {values:?}");
}

#[test]
fn mean_and_cls_pooling_disagree() {
    // They are not interchangeable: reading a mean-pooled model as CLS returns
    // right-shaped, wrongly-ranked vectors and never errors.
    let (_dir, encoder, _) = load();
    let (ids, mask) = batch(1, 4, 4);

    let mean = encoder.forward(&ids, &mask, Pooling::Mean).unwrap();
    let cls = encoder.forward(&ids, &mask, Pooling::Cls).unwrap();

    let a = mean.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    let b = cls.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    assert!(
        a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-6),
        "pooling choice had no effect: {a:?}"
    );
}

#[test]
fn different_inputs_produce_different_embeddings() {
    // Guards the degenerate case where a wiring bug makes every text collapse
    // to the same vector — which passes every shape and finiteness check.
    let (_dir, encoder, _) = load();

    let one = Tensor::from_vec(vec![4u32, 5], (1, 2), &Device::Cpu).unwrap();
    let two = Tensor::from_vec(vec![9u32, 10], (1, 2), &Device::Cpu).unwrap();
    let mask = Tensor::from_vec(vec![1u32, 1], (1, 2), &Device::Cpu).unwrap();

    let a = encoder.forward(&one, &mask, Pooling::Mean).unwrap();
    let b = encoder.forward(&two, &mask, Pooling::Mean).unwrap();
    let a = a.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    let b = b.flatten_all().unwrap().to_vec1::<f32>().unwrap();

    assert!(a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-5));
}
