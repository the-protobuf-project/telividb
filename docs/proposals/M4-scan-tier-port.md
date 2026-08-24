# Proposal · M4 + M9 — the `ScanTier` port allocates per row and cannot report a mis-prepared query

**Status:** proposal, not implemented. Changes a port trait, so CLAUDE.md
working style requires agreement first.

**Scope:** `crates/telividb-core/src/ports/scan_tier.rs`,
`crates/telividb-storage/src/tier/**`,
`crates/telividb-index/src/domain/two_tier.rs`.

M9 is folded in here because it is the same trait and the same sign-off. Fixing
both at once costs one change to every implementor instead of two.

---

## M4 · What is wrong

```rust
fn score(&self, prepared: &PreparedQuery, ordinal: Ordinal) -> Option<f32>;
```

`score` takes `&self`, so there is nowhere to put a scratch buffer. Every
implementation therefore allocates inside the innermost loop of the widest scan
in the system:

- `Int8Tier::score` — `vec![0f32; dim]` per row, then `decode_into`.
- `F16Tier::score` — the same.
- `BinaryTier::score` — `BinaryCodes::from_bytes(bytes, dim)`, which is a
  `to_vec` of the *query* codes, re-parsed for every row scanned.

`Int8Row::decode_into` exists precisely so the caller can supply the buffer, and
`ScanTier::prepare`'s own doc says the point is "the difference between a scan
that costs one transform and one that costs a transform per row." The port shape
makes that impossible to honour.

### Measured

Isolated so the only difference is where the decode buffer comes from: same
pre-parsed rows, same dot product, 200,000 rows, release build, `Metric::Dot`.

| dim | buffer per row | one buffer | saving |
|---|---|---|---|
| 128 | 108.7 ns/row | 65.2 ns/row | **40.1%** |
| 768 | 697.6 ns/row | 602.1 ns/row | **13.7%** |
| 1536 | 1384.5 ns/row | 1247.3 ns/row | **9.9%** |

The proportion falls as the vector widens because the distance computation grows
while the allocation stays a fixed `malloc` + `memset` — but it is 96 ns/row at
768 dimensions, and the coarse scan visits *every row in the field*. At ten
million rows that is roughly one second of allocator per query, on the pass
whose entire justification is being the cheap one.

Binary is not in the table because its waste is a different shape — re-parsing
the query per row — and would need its own harness. It is strictly worse than
the numbers above, not better.

---

## M9 · What is wrong

All four tiers return `None` from `score` for **two unrelated conditions**:

1. the row is absent for this field — normal, and common in a multimodal
   collection (ARCHITECTURE §4.1);
2. the `PreparedQuery` is the wrong variant for this tier, or its payload failed
   to parse — a programming error.

`two_tier::search` skips `None` rows, so a mis-prepared query scans the whole
field, matches nothing, and returns an **empty result set with no error**. That
is the same failure mode invariant 27 rules out for a locked vault: the caller
cannot tell "nothing matched" from "nothing could be computed".

It is currently unreachable through `two_tier::search`, because that function
obtains `prepared` from the same tier it then scans. It is reachable by anyone
calling `ScanTier::score` directly — which the port permits and nothing warns
against.

---

## Options

### Option A — `score_into` with a caller-supplied scratch buffer

```rust
pub trait ScanTier: Send + Sync {
    fn prepare(&self, query: &[f32], metric: Metric) -> Result<PreparedQuery>;

    /// Score one row into `scratch`, which must be at least `dim` long.
    fn score_into(
        &self,
        prepared: &PreparedQuery,
        ordinal: Ordinal,
        scratch: &mut [f32],
    ) -> Result<Option<f32>>;

    fn scratch_len(&self) -> usize;
    fn len(&self) -> usize;
}
```

- `Result<Option<f32>>` splits M9's two cases: `Err` is a mis-prepared query,
  `Ok(None)` is an absent row.
- `scratch_len` lets the caller size one buffer before the loop. Tiers that need
  no scratch (PQ, binary) return 0.

**Cost:** every implementor changes. The caller allocates once per scan.

**Risk:** a caller passing a short `scratch` — mitigated by `scratch_len` plus a
length check that returns `Err` rather than panicking.

### Option B — batched `score_range`

```rust
fn score_range(
    &self,
    prepared: &PreparedQuery,
    rows: Range<u32>,
    out: &mut Vec<Candidate>,
) -> Result<()>;
```

The tier owns the whole loop, so it allocates once internally and the port says
nothing about scratch.

**Buys:** more than Option A. The tier can hoist the query parse (binary's real
problem), decode several rows at a time, and later use SIMD across a block
without the port changing again.

**Costs:** the filter predicate has to go *into* the call, or filtering moves
back outside the scan — and post-filtering is exactly what CLAUDE.md forbids.
So the signature grows an `allowed: Option<&dyn Fn(Ordinal) -> bool>`, and the
tier becomes responsible for honouring it. That is more contract surface in the
place where getting it wrong leaks row existence.

### Option C — validate once, keep `score` as is

Fix M9 only: add

```rust
fn accepts(&self, prepared: &PreparedQuery) -> bool;
```

`two_tier::search` calls it once before the loop and errors if false. Leaves the
allocation alone.

**Cost:** one defaulted method. **Buys:** none of M4.

---

## Recommendation

**Option A**, and keep the scan loop where it is.

Option B buys more but moves filtering into every tier implementation, and
invariant 15 makes filtering a correctness boundary rather than a performance
one. A "bring your own scan tier" implementor getting the filter subtly wrong
leaks the existence and rank of rows the caller cannot see — a much worse
failure than an allocation. Option A keeps the filter in one place, in the
domain layer, where it is tested once.

If a profile later shows the per-row virtual call dominating, Option B is still
available — and by then the filter contract can be expressed as a bitmap the
tier intersects rather than a closure it must remember to call.

## What I need agreement on

1. `Result<Option<f32>>` as the return, splitting absent from mis-prepared.
2. `score_into` + `scratch_len` versus the batched `score_range`.
3. Whether `score` stays as a defaulted convenience wrapper. It keeps existing
   callers compiling, at the cost of leaving the allocating path reachable —
   my inclination is to remove it, because a slow path nobody notices is how
   this arrived.

## Tests that would land with it

- Every tier: `score_into` matches today's `score` exactly, row for row, on a
  fixed fixture. This is the correctness gate for the whole change.
- A short `scratch` returns `Err`, never a panic or a truncated read.
- A `PreparedQuery` from a *different* tier returns `Err`, not `Ok(None)` —
  the M9 regression.
- An absent row returns `Ok(None)` while a present row on the same tier returns
  `Ok(Some(_))`, so the two are distinguishable in one test.
- `two_tier::search` propagates the mis-prepared error rather than returning an
  empty result set.
- Benchmark committed alongside, reporting ns/row at 128, 768 and 1536
  dimensions, so the saving is a number rather than a claim (invariant 8).
