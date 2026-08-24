# Proposal · H2 — `present.roar` is unversioned, and is not a roaring bitmap

**Status:** proposal, not implemented. Changes an on-disk format, so CLAUDE.md
working style requires agreement first.

**Scope:** `crates/telividb-storage/src/segment/writer.rs`,
`segment/tier_reader.rs`, `segment/reader.rs`.

---

## What is wrong

Each named vector field writes a presence bitmap beside its vectors:

```rust
let mut present = vec![0u8; store.len().div_ceil(8)];
// ... set bit `row % 8` of byte `row / 8` for every present row
write_synced(dir.join("present.roar"), &present)?;
```

Two separate defects.

**It has no header.** No magic bytes, no format version. CLAUDE.md invariant 4
says every on-disk structure carries both and refuses an unknown version with a
clear error. This file refuses nothing: any byte sequence is a valid bitmap, so
a truncated, corrupted or foreign file decodes into a plausible presence set
rather than failing. The failure direction is the bad one — a row wrongly read
as present is scored from zeroed bytes and ranks as a real result.

Today `tier_reader` treats a missing or unreadable file as *all rows present*.
That is defensible for a field written before the bitmap existed, but it is
indistinguishable from a corrupted one.

**It is not a roaring bitmap.** The `.roar` extension names a format the file
does not use. It is a plain little-endian bit array, one bit per row. That
misleads about both the encoding and the cost:

| rows | bit array | roaring (all present) |
|---|---|---|
| 1M | 125 KB | tens of bytes |
| 100M | 12.5 MB | tens of bytes |

The all-present case is the common one — most fields in most collections have a
vector for every row — and it is exactly the case roaring compresses to nothing
and a bit array charges full price for.

## Why it matters now rather than later

The bitmap is read on **every tier open**, and one exists per field per segment.
A hundred-million-row collection with three named vector fields across fifty
segments pays 12.5 MB × 3 × 50 of pure allocation and page cache on a path that
learns nothing in the common case.

More importantly, this is a **format**, and formats are contracts. Adding a
header later means supporting two layouts forever. Adding one now costs a
version bump against segments that only exist on developer machines.

---

## Options

### Option A — versioned header, honest name (recommended)

Keep the bit array. Give it a 16-byte header and rename the file `present.bits`.

```text
magic(4)="EPPB" version(2) reserved(2) rows(8) then ceil(rows/8) bytes
```

- `rows` is checked against the field header's row count. A disagreement is
  `MalformedIndex`, not a guess.
- A file shorter than `16 + ceil(rows/8)` is `Truncated`.
- Absent file still means "all present", but now that is a *decision* recorded
  in one place rather than an accident of `unwrap_or_default()`.

**Cost:** 16 bytes per field. One format version bump. Rename touches three
call sites.

**Does not fix:** the 12.5 MB at 100M rows.

### Option B — actual roaring bitmap

Replace the encoding with roaring, with the same versioned header.

**Cost:** a dependency (`roaring` is pure Rust, no build script — it does not
touch invariant 1) or an implementation. Roaring's own serialized format is
already versioned and stable, so the header wraps rather than replaces it.

**Buys:** the all-present case collapses to a run container of a few bytes, and
the sparse case — a field populated for 2% of rows — collapses to an array
container. Both are the shapes that actually occur.

**Risk:** presence is checked per row during a scan
(`is_present(row)` in `tier_reader`). A bit array answers that in one shift; a
roaring bitmap answers it in a container lookup. For a **sequential** full-field
scan that is fine — iterate the bitmap alongside the rows rather than probing it
per row — but it means the scan loop changes shape, and a random-access probe
(graph traversal reaching an arbitrary ordinal) gets slower.

### Option C — header now, roaring later

Option A, but reserve the `codec` byte in the header so a later version can
switch the payload encoding without a second rename.

---

## Recommendation

**Option C.** The header is the invariant-4 violation and is cheap and
uncontroversial; the encoding change has a real trade-off on the random-access
path that deserves a benchmark before it is made. Reserving the encoding byte
means making that call later costs a version bump and nothing else.

Concretely:

```text
magic(4)="EPPB"  version(2)=1  encoding(1)=0 (bit array)  reserved(1)
rows(8)  payload_len(4)  reserved(4)     = 24 bytes
```

`encoding = 1` is roaring, unimplemented until there is a measurement.

## What I need agreement on

1. Header or not (invariant 4 says yes; confirming because it is a format).
2. Rename `present.roar` → `present.bits`. It is a rename of a file inside a
   segment directory, not an API change, but it invalidates existing segments.
3. Whether to reserve the encoding byte for roaring, or defer the whole
   question.
4. Whether "file absent means all rows present" stays. It is the compatibility
   shim for pre-bitmap segments; if there are none in the world, dropping it
   makes a missing file an error, which is the safer default.

## Tests that would land with it

- Round-trip: write a field with a mixed presence pattern, reopen, assert every
  row's presence matches.
- A file with the wrong magic is refused.
- A file declaring a newer version is refused.
- A file whose `rows` disagrees with the field header is refused.
- A truncated payload is `Truncated`, not a short read that reports absence.
- Golden fixture: a committed segment that must keep opening across changes.
