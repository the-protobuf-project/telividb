use super::*;

#[test]
fn the_same_seed_gives_the_same_sequence() {
    let mut a = SplitMix64::new(42);
    let mut b = SplitMix64::new(42);
    for _ in 0..100 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn different_seeds_diverge() {
    let mut a = SplitMix64::new(1);
    let mut b = SplitMix64::new(2);
    assert_ne!(a.next_u64(), b.next_u64());
}

#[test]
fn floats_stay_strictly_inside_zero_and_one() {
    // Zero would make ln() infinite and the level unbounded.
    let mut r = SplitMix64::new(7);
    for _ in 0..10_000 {
        let x = r.next_f64();
        assert!(x > 0.0 && x < 1.0, "out of range: {x}");
    }
}

#[test]
fn levels_are_finite_and_mostly_zero() {
    let factor = 1.0 / 16f64.ln();
    let mut r = SplitMix64::new(3);
    let mut zeros = 0;
    let n = 10_000;
    for _ in 0..n {
        let level = r.level(factor);
        assert!(level < 64, "level {level} is implausible");
        if level == 0 {
            zeros += 1;
        }
    }
    // With m=16 roughly 15/16 of nodes should land on layer zero.
    let ratio = zeros as f64 / n as f64;
    assert!(ratio > 0.85 && ratio < 0.99, "layer-zero ratio {ratio}");
}
