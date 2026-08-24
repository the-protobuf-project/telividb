# Branch review — `feat/adding-telemetry`

Reviewed: `main...HEAD`, 170 files, ~14.8k added lines.
Gates at time of review: `cargo clippy -D warnings` clean · `cargo test --workspace
--all-features` green · `check-len` 168 files OK · `check-docs` 1209 comments OK.

Everything below is a defect the existing gates do not catch. Items marked
**[verified]** were reproduced with a failing test during the review.

---

## Highest severity — the observability pipeline is installed nowhere **[verified]**

These two land above everything else: they are silent, total, and they make the
rest of the telemetry findings academic.

### T1 · No `metrics` recorder exists in the process, so every metric is discarded
`crates/telividb-telemetry/Cargo.toml:20`, `src/init.rs:158`

The branch drops `metrics-exporter-prometheus` — the only crate that installed a
`metrics` recorder — and replaces it with `telemetry-rs`. Verified against the
pinned checkout (`tag = v1.1.0`, `884f94d`):

- `telemetry-rs` does not depend on the `metrics` crate at all. Its `Cargo.toml`
  matches on `metrics` only for `opentelemetry`'s feature and for an example
  binary named `metrics`.
- `grep set_global_recorder` returns nothing in `telemetry-rs`, and nothing in
  `crates/`.

The `metrics` facade with no recorder installed is a no-op by design. So every
emission site in `telividb-storage` and `telividb-index` — search latency, WAL
commit duration, torn-tail recoveries, segment seal time, index build time, live
segment and row gauges — records nothing, in every configuration including
`--otlp`. `describe_all()` registers descriptions with no registry.

README advertises `--otlp … # export logs, traces and metrics`. The metrics half
is false.

**Fix:** install a recorder in the `subscriber` feature — either a
`metrics`→OpenTelemetry bridge feeding `telemetry-rs`'s meter, or reinstate a
Prometheus recorder. Add a test that asserts a recorder is installed after
`Telemetry::install`.

### T2 · No `tracing` subscriber is installed unless `--otlp` is passed
`crates/telividb-telemetry/src/init.rs:104-108`

`telemetry-rs` calls `tracing::dispatcher::set_global_default` from exactly one
place — `init_tokio_tracing`, at `telemetry/src/lib.rs:237`. That call is gated:

```rust
let tracing_tracer = telemetry.as_ref().and_then(|t| t.get_tracer(&name));
if let Some(tracer) = tracing_tracer.clone() {
    tracing::init_tokio_tracing(tracer)?;
}
```

`init.rs` only reaches `.with_tracing()` inside `if let Some(addr) = config.otlp`,
so with no collector configured there is no tracer, no subscriber, and every
`tracing::info_span!` / `tracing::info!` / `tracing::error!` in the workspace is a
no-op. The previous code installed `tracing_subscriber::fmt()` unconditionally.

That is the actual cause of the silence the `logger::info!` calls in
`serve.rs` and `services/collection.rs` were added to paper over — including
`tracing::error!(%error, "malformed index")` in `error.rs`, which is the one
diagnostic an operator most needs.

**Fix:** install a console subscriber whenever the OTLP path is off, so spans and
events always land somewhere. Then reconsider the per-request `logger::info!`
calls (see T3).

### T3 · Per-request logging on the RPC path
`crates/telividb-server/src/services/collection.rs:34,60,91`

`list_collections` and `create_collection` emit an unconditional `logger::info!`
per call, at a level that is live in `Environment::Production`. `ListCollections`
was deliberately made to always succeed, so it is the natural liveness probe — a
polling client turns it into unbounded synchronous console volume on the tonic
handler thread. Once T2 is fixed, the sampled span carries this instead.

Also: `get_collection`, `list_collections` and `delete_collection` hold
`span.enter()` guards across `async fn` bodies. Benign only because none of them
awaits yet; the first `.await` added under one of these leaks the span — and the
collection name on it — onto whatever task the executor runs next. Use
`.instrument(span)` or `#[tracing::instrument]` before the catalogue lands.

---

## Fix workstreams

Each workstream is a self-contained agent task. File sets are disjoint except
where noted, so WS-1, WS-4 and WS-5 can run concurrently.

**WS-2 and WS-3 change an on-disk format and a port trait respectively.**
CLAUDE.md working style requires proposing those before implementing — an agent
must return a proposal, not a diff, for the items flagged `PROPOSE`.

| WS | Scope | Files |
|----|-------|-------|
| 1 | HNSW build correctness | `crates/telividb-index/src/adapters/hnsw/**`, `crates/telividb-index/tests/hnsw_parallel.rs` |
| 2 | Segment durability & format | `crates/telividb-storage/src/segment/**`, `crates/telividb-storage/src/format/quantize/**` |
| 3 | Scan-tier hot path | `crates/telividb-core/src/ports/scan_tier.rs`, `crates/telividb-storage/src/tier/**`, `crates/telividb-index/src/domain/two_tier.rs` |
| 4 | Telemetry & server wiring | `crates/telividb-telemetry/**`, `crates/telividb-server/**` |
| 5 | Tooling, CI, docs | `.cargo/config.toml`, `tools/xtask/**`, `.github/workflows/ci.yml`, `CLAUDE.md` |

Ordering: WS-1 → (WS-2 → WS-3) with WS-4 and WS-5 in parallel. WS-2 before WS-3
because WS-3's trait change is easier to land once the format questions settle.

---

## WS-1 — HNSW build correctness

### C1 · Batched build orphans the entire first batch **[verified]**
`adapters/hnsw/batched.rs:102-107`

`find_candidates` returns `Vec::new()` when `graph.entry()` is `None`. During
batch 0 the snapshot graph is empty for *every* node in the batch, so all of
them are pushed with zero edges. They are never linked afterwards, because
nothing can find them.

Measured on a 3,000-row clustered corpus, `Metric::Cosine`, default params:

| `batch_size` | layer-0 orphans | recall@10 |
|---|---|---|
| 1 | 0 | 1.0000 |
| 32 | 31 | 0.9900 |
| 64 | 63 | 0.9725 |
| 256 | 256 | 0.8100 |
| 512 | 511 | 0.5675 |

Orphan count is exactly `batch_size - 1` in every case.

**Fix:** insert the first batch sequentially (or, more simply, seed the graph
with row 0 before entering the batch loop), so every subsequent batch searches a
non-empty snapshot. Keep the row-order apply phase — determinism must survive.

### C2 · The batching trade-off table is measuring C1
`adapters/hnsw/params.rs:37-55`

The doc attributes the recall curve to "nodes within a batch cannot link to each
other" and to Amdahl. The measurements above show the curve is dominated by C1
instead. `batch_size: 1` was chosen from that table.

**Fix:** after C1, re-measure and rewrite the table and the rationale. Revisit
whether `1` is still the right default.

### C3 · An absent row at the head of a field orphans present rows **[verified]**
`adapters/hnsw/build.rs:82-87`, `adapters/hnsw/graph.rs:65-68`

`build` pushes `push_node(0)` for a row with no vector for this field.
`push_node` makes the first node the entry point unconditionally, so an absent
row becomes the entry. `insert` then computes `distance_to(store, …, entry)`,
gets `None`, and returns early — leaving the node unlinked. This continues until
some node draws a level above 0 and takes over as entry.

Measured, 2,000 rows, absent prefix at the head:

| absent prefix | orphaned *present* rows | recall@10 |
|---|---|---|
| 1 | 10 | 0.9950 |
| 5 | 6 | 0.9950 |
| 20 | 20 | 0.9900 |
| 100 | 12 | 0.9950 |

**Fix:** never let an absent row become the entry point — track the entry
separately from `push_node`, or have `build` skip the level draw for absent rows
and choose the first *present* node as entry.

### C4 · An unreachable index reports "no results" rather than an error
`adapters/hnsw/mod.rs:134-136`

`search` returns `Ok(Vec::new())` when the entry row has no vector. A caller
cannot distinguish an empty collection from a broken index. Same shape as the
`complete = false` reasoning in CLAUDE.md invariant 27.

**Fix:** return `Error::MalformedIndex` when the entry point cannot be scored.

### L4 · The tests that should have caught C1
`tests/hnsw_parallel.rs:46-65`

- `batching_costs_recall_and_the_cost_grows_with_batch_size` asserts only
  `sequential >= wide`. It never puts a floor under `wide`, so 0.5675 passes.
- `a_batch_larger_than_the_corpus_still_builds` passes `batch_size: 100_000`
  against 500 rows. `build` requires `store.len() > params.batch_size` to take
  the batched path, so this test runs the **sequential** builder. It has never
  exercised batching.

**Fix:** assert an absolute recall floor per batch size; make the
larger-than-corpus test actually reach `build_batched`; add a test asserting
zero layer-0 orphans among present rows.

### L6 · `decode` discards what the header stores
`adapters/hnsw/serialize.rs:73-75`

`entry`, `max_level` and `edges` are read into `_`-prefixed bindings and
validated against nothing. The graph is reconstructed by replaying `push_node`,
which happens to reproduce the same entry — so a file whose header disagrees
decodes silently into a different graph.

**Fix:** verify the decoded graph's entry, max level and edge count against the
header; error on mismatch.

### M5 · A full-corpus allocation per query
`adapters/hnsw/mod.rs:143`

`VisitedSet::with_capacity(store.len())` allocates and zeroes `4 * rows` bytes on
every `search` call. `visited.rs:1-10` documents this exact pattern as the trap
that made builds quadratic.

**Fix:** hold a pooled/thread-local `VisitedSet` on the index, or take a scratch
argument.

---

## WS-2 — Segment durability & format

### H1 · Sidecar files are not fsynced before the segment is published
`segment/writer.rs:114`, `segment/codes.rs:61-69`

`raw.bin` and `header.bin` are `sync_all()`d. `present.roar` (`fs::write`) and
`codebook.pq` (`fs::write`) are not, and the temp directory itself is never
synced before `fs::rename`. A crash can therefore publish a segment — the rename
is made durable by the parent-directory sync — whose presence bitmap or codebook
is missing or zero-length.

**Fix:** fsync every file, then fsync the temp directory, then rename. Add a
crash-consistency test alongside the existing `crash_consistency` suite.

### H2 · `present.roar` is unversioned, and is not a roaring bitmap · `PROPOSE`
`segment/writer.rs:101-114`, `segment/tier_reader.rs:45-51`

The file has no magic bytes and no format version, which is a direct violation of
CLAUDE.md invariant 4. It is also a plain little-endian bit array despite the
`.roar` extension, so the name misleads about the format and about the cost — an
all-present 100M-row field costs 12.5 MB here versus a few bytes as roaring.

**Fix (propose first):** either give it a versioned header and rename it to match
what it is, or make it an actual roaring bitmap. Format change → needs sign-off.

### H3 · Opening a scan tier reads the whole full-precision file
`segment/tier_reader.rs:22-23`

```rust
let header_bytes = std::fs::read(dir.join("raw.bin"))?;
let header = FieldHeader::decode(&header_bytes[..FIELD_HEADER_BYTES.min(header_bytes.len())])?;
```

The entire `raw.bin` is read into memory to parse a 32-byte header and then
dropped. Line 29 additionally reads all of `codes.bin` into RAM. Both violate
invariant 3 — "Never copy a whole segment to search it."

**Fix:** read only `FIELD_HEADER_BYTES` from `raw.bin`; back the codes with the
`BlockReader` port rather than `fs::read`.

### H4 · A PQ codebook trained on nothing succeeds
`format/quantize/pq/codebook.rs:64-97`, `pq/kmeans.rs:49-70`

With zero training vectors, `seed_centroids` returns all zeros and `train` breaks
out before any `update`. `PqCodebook::train` returns `Ok` with a degenerate
codebook; every row then encodes to code 0 and the tier ranks nothing — silently.
The same path degrades quietly whenever the training set is smaller than
`CENTROIDS` (256).

**Fix:** error on an empty training set; warn or error below 256 vectors.

### H5 · `from_codes` panics on short input through a public API
`tier/int8.rs:48`, `tier/f16.rs:45`, `tier/binary.rs:48`

Each slices `&codes[start..]` without checking `start` against `codes.len()`,
then constructs a `Truncated` error in the `ok_or` arm it can never reach — the
slice panics first. `tier_reader` guards the length before calling, but these are
`pub fn` taking untrusted bytes.

**Fix:** `codes.get(start..)` and return the `Truncated` error the code already
means to return.

### L12 · Temp-directory names can collide
`segment/writer.rs:29`

`final_path.with_extension("building")` *replaces* an existing extension, so
`seg.1` and `seg.2` both produce `seg.building`.

**Fix:** append rather than replace.

### L14 · Unchecked multiplication in a function documented as checking everything
`format/quantize/pq/serialize.rs:67-74`

`expected * 4` is computed without `checked_mul` in a function whose doc says
"validating every declared length before it is used". Safe on 64-bit for
u32-bounded inputs; not on 32-bit.

### L15 · `Codec::None` still creates an empty `codes.bin`
`segment/codes.rs` — guarded by the caller, but the function writes the file
before checking. Low impact; a stray empty file in a sealed segment.

---

## WS-3 — Scan-tier hot path

### M4 · Every tier allocates per row scored · `PROPOSE`
`tier/int8.rs:95`, `tier/f16.rs:~88`, `tier/binary.rs:~100`

`ScanTier::score` takes `&self`, so there is nowhere to put a scratch buffer.
Int8 and F16 therefore do `vec![0f32; dim]` for **every row scanned**, and Binary
re-parses the query with `BinaryCodes::from_bytes` (a `to_vec`) for every row.
This is the wide-cheap-scan inner loop; `Int8Row::decode_into` exists precisely to
avoid it, and `ScanTier::prepare`'s own doc says the point is "the difference
between a scan that costs one transform and one that costs a transform per row."

**Fix (propose first):** change the port — a `score_into(&self, prepared,
ordinal, scratch: &mut [f32])`, or a batched `score_range`. Port change → needs
sign-off.

### M6 · The coarse scan materializes and full-sorts the whole corpus
`domain/two_tier.rs:76-95`

Every scored row is pushed into a `Vec`, then `sort_unstable_by` runs over all of
them, then it is truncated to `want`. For the scan that is supposed to be the
cheap one, that is an O(n) allocation and an O(n log n) sort per query.

**Fix:** bounded max-heap of size `want`.

### M8 · Unchecked index in `PqTier::score`
`tier/pq.rs:~130` — `distances[sub * CENTROIDS + code as usize]`.
`PreparedQuery::table` is public and accepts an arbitrary `distances` vector, so a
short table panics. Use `.get()`.

### M9 · "Row absent" and "wrong prepared state" are the same return
All four tiers return `None` from `score` both for an absent row and for a
prepared query of the wrong variant or a failed parse. `two_tier::search` skips
`None` rows, so a mis-prepared query returns an empty result set with no error.

**Fix:** distinguish them — `Result<Option<f32>>`, or validate the prepared state
once in `prepare`/at the top of the scan rather than per row.

---

## WS-4 — Telemetry & server wiring

> Note: T1 means none of M1/M2 is observable today. They are still wrong and
> become live the moment a recorder is installed — fix T1 first, then these.

### M1 · Every metric is registered as a histogram
`telemetry/src/init.rs:158-162`

`describe_all` loops `metrics_names::ALL` calling `describe_histogram!`. That list
includes counters (`SEARCH_INCOMPLETE`, `WAL_BYTES`, `WAL_TORN_RECOVERIES`,
`POLICY_DENIED`, `JOB_RECORDS`) and gauges (`SEGMENTS_LIVE`, `ROWS_LIVE`,
`ROWS_TOMBSTONED`). The module doc says this exists so `/metrics` is
self-documenting; as written it mislabels a third of it.

**Fix:** carry the instrument kind in `ALL` and dispatch to
`describe_histogram!` / `describe_counter!` / `describe_gauge!`.

### M2 · Per-operation writes to process-wide gauges
`storage/src/compact/compact.rs:116`, `storage/src/segment/writer.rs:142`

`compact_field` ends with `gauge!(ROWS_TOMBSTONED).set(0.0)` — compacting one
field of some segments does not make the database's tombstone count zero.
`SegmentWriter::finish` sets `ROWS_LIVE` to the row count of the segment just
written, not of the database.

**Fix:** these gauges belong to whatever owns the manifest. Remove them from the
per-operation paths.

### M3 · Telemetry install failures are swallowed into a void
`server/src/serve.rs:30-33`

Every `Err` from `Telemetry::install` is treated as "already installed" and logged
with `tracing::debug!` — through a facade that, in the failure case, usually has no
subscriber behind it. An unwritable `--mcap` path or a bad OTLP endpoint starts a
server with no telemetry and no diagnostic anywhere.

**Fix:** distinguish "already installed" from a genuine failure, and report the
latter on stderr before the pipeline exists.

### T5 · An IPv6 OTLP address is silently malformed **[verified]**
`crates/telividb-telemetry/src/init.rs:106`

`init.rs` splits a `SocketAddr` with `with_otlp(addr.ip().to_string(), addr.port())`.
Upstream rejoins them at `telemetry/src/lib.rs:696` with
`format!("{}:{}", host, port)` — no brackets. So `--otlp [::1]:4317` parses fine,
then becomes the endpoint `::1:4317`, which is not a valid URI. The exporter
either fails to build — and that failure is swallowed by M3 — or never connects,
while `serve.rs:76` prints "exporting logs, traces and metrics to [::1]:4317".

**Fix:** pass `addr.to_string()` as the host (it brackets IPv6 correctly) with the
port, or reject IPv6 with a clear error.

### T6 · `telemetry.toml` changes nothing **[verified]**
`telemetry.toml`, and CLAUDE.md rule 41

`OTLPOptions::default()` already has `enabled: false`
(`telemetry/src/options/opentelemetry.rs:178`), and the effective condition is
`telemetry.enabled && otlp.enabled`. With or without the file, OTLP is off. The
`[service]` block is then unconditionally overwritten by `.with_service()` /
`.environment()`, and the file is discovered relative to the process CWD, so a
deployed binary never reads it anyway.

CLAUDE.md rule 41 records the false premise as an invariant — "the OTLP pipeline
is on unless `telemetry.toml` disables it, which is why that file exists at the
repository root". A future reader will delete the file expecting exporting to
turn on. `version = "0.1.0"` in it also duplicates the workspace version and is
dead, so it will drift silently at the next bump.

**Fix:** delete the file and correct rule 41, or make it load-bearing.

### T7 · `ServerConfig::environment` is a `String` validated only at the CLI
`crates/telividb-server/src/config.rs:23`, `serve.rs:117-124`

`Environment` is already re-exported from `telividb-telemetry`. Carrying it as a
`String` means an embedded caller writing
`ServerConfig { environment: "prod".to_owned(), ..ServerConfig::at(addr) }` —
the pattern `tests/wiring.rs` already uses — bypasses `args::parse` entirely and
falls through `environment_of`'s `_ => Environment::Development`. A production
deployment then logs at debug while printing "telemetry: environment prod".

Also violates CLAUDE.md's "No stringly-typed config lookups scattered through the
code."

**Fix:** type the field as `Environment`; `args::parse` does the string→enum
conversion once, and the silent fallback disappears with it.

### T8 · `mcap_path` round-trips through `Display`, which is lossy
`crates/telividb-server/src/serve.rs:22`

`config.mcap_path.as_ref().map(|p| p.display().to_string())` rewrites a non-UTF-8
path with U+FFFD rather than rejecting it, so the stack opens a different path
than the one the startup line reports. Root cause is
`TelemetryConfig::mcap_path: Option<String>` where a `PathBuf` belongs.

### T9 · The only test that exercised `Telemetry::install` was deleted
`crates/telividb-telemetry/src/init_test.rs`

The rewrite removed `a_bad_filter_directive_is_rejected` and replaced it with
nothing. What survives asserts on `TelemetryConfig::default()` fields and the pure
`should_sample`. Nothing covers `install()`, the otlp/mcap/environment → builder
mapping, `describe_all()`, or `TelemetryError::Install`. T1, T2 and T5 all live in
code no test touches — which is why they landed.

### L7 · Dead error variants
`server/src/error.rs` — `Error::Bind` and `Error::Telemetry` are never
constructed. Either construct them (the bind failure currently surfaces as
`Transport`) or remove them.

### L8 · A signal-handler failure shuts the server down
`server/src/serve.rs:131-135` — `shutdown()` resolves whether `ctrl_c()`
succeeds or errors, so a failure to install the handler triggers immediate
graceful shutdown. Return `std::future::pending()` on error.

---

## WS-5 — Tooling, CI, docs

### L1 · `cargo dev` is broken **[verified]**
`.cargo/config.toml:13` passes `--log debug,h2=info,tower=info`. `args::parse`
has no `--log` flag — CLAUDE.md rule 41 made verbosity a function of the
environment. Running `cargo dev` prints `telividb: unknown flag --log` and exits
non-zero.

**Fix:** `dev = "run --package telividb-server --bin telividb -- --environment development"`.

### L2 · `cargo xtask check-layers` does not exist
CLAUDE.md's Commands section lists it and rule 14 says the layering rule is
enforced by it. `tools/xtask/src/main.rs` has no such task.

**Fix:** implement it, or strike the claim from CLAUDE.md. Implementing is the
better answer — nothing else enforces invariant 14.

### L3 · `check-docs` does not do what it is advertised to do
`tools/xtask/src/check_docs.rs`, `main.rs:40`

The usage text says "fail on any public item without a doc comment" and CLAUDE.md
repeats it. The implementation only scans lines that *already* start with `/// `
and flags ones under three words. An undocumented item is invisible to it. The
compiler's `deny(missing_docs)` covers publicly-reachable items only — e.g.
`kmeans::nearest_centroid` is `pub`, undocumented, and caught by neither.

**Fix:** either detect undocumented `pub` items, or correct both strings.

### T4 · The `no C toolchain` guard can never fail **[verified]**
`.github/workflows/ci.yml:115-135`

```sh
offenders=$(cargo tree … | grep -E '^(cc|cmake|bindgen) ')
unexpected=$(cargo tree … --invert cc … | grep -E '^(lz4-sys|lz4|mcap|telemetry) ')
if [ -n "$offenders" ] && [ -z "$unexpected" ]; then … exit 1; fi
```

`unexpected` is populated from evidence of the **expected** lz4→mcap→telemetry
path, not from an unexpected one. That path is permanently in the tree, so
`unexpected` is never empty and the `if` never fires. Run locally on this branch:

- `offenders` → `cc v1.4.4`
- `unexpected` → `lz4 v1.28.1`, `lz4-sys v1.11.1`, `mcap v0.24.0`, `telemetry v1.0.0`

So the job prints "no unexpected native build dependencies" and exits 0
regardless. Adding `openssl-sys`, `zstd-sys` or any `cmake`/`bindgen` dependency
passes CI green — the invariant-1 carve-out is unguarded, not narrowed, which is
the opposite of what the comment above it claims.

`--invert cc` also means `cmake` and `bindgen` are no longer checked against the
carve-out at all, even though `offenders` still greps for them.

**Fix:** compute the offending paths directly — for each of `cc`, `cmake`,
`bindgen`, take `cargo tree --invert <pkg>` and fail if any root other than the
sanctioned lz4/mcap/telemetry chain appears.

### L5 · The recall gate does not cover the batched builder
`.github/workflows/ci.yml:183-199` runs `hnsw_recall`, `hnsw_distribution`,
`hnsw_persist`, `codec_recall`, `two_tier_recall`. `hnsw_parallel` is absent. Add
it once WS-1 lands.

### L16 · Default-feature tests are no longer run anywhere
`.github/workflows/ci.yml:74`

`cargo test --workspace --target …` was replaced by `--all-features` only, with a
comment claiming `--all-features` "is a superset of the default run". It is not:
`--all-features` changes which `#[cfg(feature = …)]` code compiles. In particular
`telividb-telemetry` *without* `subscriber` is the configuration every library
crate links, and it is now only compile-checked, never executed.

### L9 · `CompactionResult::rows_reclaimed` contradicts the code
`storage/src/compact/compact.rs:28` says "Rows dropped because they were
tombstoned **or absent**"; lines 101-104 write absent rows and count them as
written. Only tombstoned rows are reclaimed.

### L10 · `compact_field` ignores the two parameters it takes
`storage/src/compact/compact.rs:51-56` — `_schema: Fingerprint` and `_codec:
Codec` are accepted and dropped, so a compacted field silently loses its scan
tier. Either use them or remove them from the signature.

### L11 · Duplicated doc line
`index/src/adapters/memory_store.rs:29-30` — the same `///` line twice.

### L13 · Compaction plans have no input cap
`storage/src/compact/plan.rs:66-79` — every tombstone-heavy segment goes into one
plan. With hundreds of segments that is a single unbounded rewrite.

---

## Appendix — launching the fix agents

Each block is a complete agent prompt. `isolation: "worktree"` is recommended so
concurrent agents cannot collide, and so a bad run is discarded rather than
unwound.

**Order.** T1/T2 first — until a recorder and a subscriber exist, no telemetry
fix in WS-4 can be verified by observation. Then WS-1 (largest recall win), then
WS-5 (cheap, unblocks CI trust), then WS-2 → WS-3.

**Every agent must:**
- Read `CLAUDE.md` first and treat its numbered invariants as binding.
- Keep every file under 200 lines; tests go in a sibling `*_test.rs`.
- Finish with `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace --all-features`, `cargo fmt --all`,
  `cargo xtask check-len`, `cargo xtask check-docs` — all green.
- Add a regression test for every item it fixes. For WS-1, report recall@10
  before and after; "it builds" is not a result (invariant 8).
- **Stop and return a written proposal** — not a diff — for anything marked
  `PROPOSE`. Those touch an on-disk format or a port trait.

### Agent T — restore the observability pipeline
> Fix T1, T2, T3, T5, T6, T7, T8, T9 in `docs/REVIEW_FIXES.md`. The pinned
> `telemetry-rs` checkout is at
> `~/.cargo/git/checkouts/telemetry-a5c941e7a7a5ccca/884f94d/telemetry-rs`; read
> it rather than guessing at its behaviour. The core of the task is that no
> `metrics` recorder and no `tracing` subscriber are installed in any
> configuration, so every instrumentation call in the workspace is a no-op. Land
> a test that fails if either is missing after `Telemetry::install`.

### Agent 1 — HNSW build correctness
> Fix C1, C2, C3, C4, L4, L6, M5 in `docs/REVIEW_FIXES.md`. C1 and C3 are
> reproduced there with measured recall tables; reproduce them as failing tests
> first, then fix. After C1, re-measure the `batch_size` table in `params.rs` and
> rewrite both the table and its explanation — the current one attributes the
> curve to Amdahl when it is a bug. Report recall@10 per batch size before and
> after.

### Agent 5 — tooling, CI and docs
> Fix T4, L1, L2, L3, L5, L9, L10, L11, L13, L16 in `docs/REVIEW_FIXES.md`. T4
> and L1 are verified broken. For L2, implement `cargo xtask check-layers` —
> nothing currently enforces invariant 14. For L3, either make `check-docs`
> detect undocumented `pub` items or correct the two places that claim it does.

### Agent 2 — segment durability and format
> Fix H1, H3, H4, H5, L12, L14, L15 in `docs/REVIEW_FIXES.md`, and return a
> written proposal for H2 without implementing it. H1 needs a crash-consistency
> test in the style of the existing `crash_consistency` suite.

### Agent 3 — scan-tier hot path
> Fix M6, M8, M9 in `docs/REVIEW_FIXES.md`. Return a written proposal for M4 —
> it changes the `ScanTier` port, which is a design decision requiring sign-off.
> Include a benchmark showing the per-row allocation cost the proposal removes.
