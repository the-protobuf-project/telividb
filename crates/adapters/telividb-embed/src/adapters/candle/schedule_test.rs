use super::*;

/// Every input appears exactly once, across all batches.
fn covers_everything(batches: &[Batch], count: usize) -> bool {
    let mut seen: Vec<usize> = batches.iter().flatten().copied().collect();
    seen.sort_unstable();
    seen == (0..count).collect::<Vec<_>>()
}

#[test]
fn every_text_is_scheduled_exactly_once() {
    // The contract the caller depends on: one vector per input. A dropped or
    // duplicated index would silently misalign every result after it.
    let lengths = vec![10, 500, 3, 2000, 47, 8, 900];
    let batches = plan(&lengths, 4096, 64);
    assert!(covers_everything(&batches, lengths.len()));
}

#[test]
fn similar_lengths_end_up_together() {
    // The whole point. With sorting, the 2000-token text must not share a
    // batch with the 3-token one, because the short text would then be
    // computed at 2000 tokens.
    let lengths = vec![3, 2000, 4, 1900, 5];
    let batches = plan(&lengths, 4096, 64);

    let batch_of = |i: usize| batches.iter().position(|b| b.contains(&i)).unwrap();
    assert_ne!(batch_of(0), batch_of(1), "a 3-token text batched with a 2000-token one");
    assert_eq!(batch_of(1), batch_of(3), "1900 and 2000 should share a batch");
}

#[test]
fn the_token_budget_is_respected() {
    // rows * padded_length is what drives memory and time, so it is what the
    // budget has to bound.
    let lengths = vec![100; 20];
    let batches = plan(&lengths, 1000, 64);

    for batch in &batches {
        let padded = batch.iter().map(|i| lengths[*i]).max().unwrap();
        assert!(
            batch.len() * padded <= 1000,
            "batch of {} x {padded} exceeds the budget",
            batch.len()
        );
    }
}

#[test]
fn the_row_cap_is_respected() {
    // At tiny lengths the budget alone would allow thousands of rows, where
    // per-row overhead starts to dominate.
    let lengths = vec![1; 500];
    let batches = plan(&lengths, 100_000, 32);
    assert!(batches.iter().all(|b| b.len() <= 32));
    assert!(covers_everything(&batches, lengths.len()));
}

#[test]
fn a_text_longer_than_the_whole_budget_still_runs() {
    // Refusing it would make a single long document unembeddable, which is a
    // worse failure than one oversized batch.
    let lengths = vec![10_000, 5];
    let batches = plan(&lengths, 1000, 64);

    assert!(covers_everything(&batches, 2));
    assert!(batches.iter().any(|b| b == &vec![0]));
}

#[test]
fn an_empty_input_plans_nothing() {
    assert!(plan(&[], 4096, 64).is_empty());
}

#[test]
fn a_zero_length_text_does_not_divide_by_zero_or_loop() {
    // An empty string tokenizes to very little, and a zero would make the
    // `rows * padded` product zero — allowing an unbounded batch.
    let lengths = vec![0, 0, 0];
    let batches = plan(&lengths, 4096, 2);
    assert!(covers_everything(&batches, 3));
    assert!(batches.iter().all(|b| b.len() <= 2));
}
