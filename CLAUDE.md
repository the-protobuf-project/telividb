# CLAUDE.md

Guidance for Claude Code when working in this repository.

> **New to this repo? Read [`AGENT_START.md`](./AGENT_START.md) first.** It holds the
> architecture, the phased roadmap, and the explicit list of what is and is not in scope.
> This file is the *working rules*; that file is the *plan*.

---

## What this is

**episteme** — a single-node, embedded-or-served **vector database** written in Rust, with a
gRPC API as the primary interface. Two things distinguish it from the field:

1. **Bring your own embedding model.** Models load from **GGUF** files, described by a config
   file. Inference runs on whatever hardware is present — Metal, CUDA, Jetson, Intel, plain CPU —
   without recompiling the database.
2. **Bring your own search algorithm.** ANN indexes are pluggable behind a trait. Flat, HNSW and
   IVF-PQ ship in-tree; a user-supplied index is a first-class citizen.

Plan A is the vector store. Plan A1.1 layers a property graph over the same storage so the
system becomes a **graph + embedding database** for GraphRAG-style retrieval.

MCP access already exists on the user's side. **Do not build, scaffold, or "helpfully add" an
MCP server here** unless explicitly asked. gRPC is the contract; MCP consumes it.

---

## Non-negotiable invariants

Violating any of these is a bug, not a style preference.

1. **The database is 100% Rust.** No C or C++ in `episteme-core`, `-storage`, `-index`,
   `-distance`, `-query`, `-graph`, `-server`. The *only* sanctioned FFI is the optional
   llama.cpp embedding backend, quarantined in its own crate behind a non-default feature.
   A default `cargo build` must need no C toolchain, no CMake, no CUDA SDK.
2. **Sealed segments are immutable.** Once a segment is sealed, its `vectors.bin`, `ids.bin` and
   `index.*` files are never written again. Mutation happens by writing a new segment plus a
   tombstone bitmap. This is what makes lock-free concurrent reads and `mmap` safe — do not
   "optimize" it away.
3. **Zero-copy on the read path.** Vectors are fixed-stride and 64-byte aligned so an mmap'd
   region casts directly to `&[f32]` / `&[u8]`. Never `Vec<Vec<f32>>`. Never copy a whole segment
   to search it.
4. **Every on-disk structure is versioned.** Magic bytes + format version in every file header.
   Refuse to open unknown versions with a clear error; never guess.
5. **No page faults on an async executor thread.** mmap reads block. All search and IO work runs
   on a dedicated blocking pool, never directly inside a `tonic` handler's async context.
6. **The index never touches files.** Indexes talk to a `VectorStore` trait. Storage layout and
   search algorithm evolve independently — that separation is the whole point of "bring your own
   search algorithm."
7. **Distance kernels are runtime-dispatched.** Detect AVX2 / AVX-512 / NEON at startup. A scalar
   fallback must always exist and must be correct. `target-cpu=native` is never required to run.
8. **Correctness is measured against brute force.** Any ANN index change must report recall@k
   versus the flat index on a fixed dataset. "It's faster" without a recall number is not a
   result.
9. **External IDs are the only portable identity.** Internal ordinals are segment-local and must
   never appear in an archive, an API response, or either end of an exported edge. Anything that
   leaks an ordinal across a process boundary is a bug.
10. **Bulk operations are durable jobs, never blocking RPCs.** They return a job handle, checkpoint
    their progress, resume after restart, and are cancellable. No bulk path may assume it fits in
    a gRPC deadline.
11. **Partial failure is the default for bulk work.** One bad record out of ten million does not
    fail the run — it goes to the reject file with enough context to be fixed and re-imported.
    All-or-nothing is opt-in (`on_error = ABORT`), never the default.
12. **Each named vector field is bound to one embedding model identity.** Name, GGUF hash, dim,
    pooling, normalization, prefix scheme, and the query-side encoder. Ingesting vectors from a
    different model into a field is rejected unless explicitly overridden. Mixed provenance
    degrades recall silently, which is the worst failure mode available.
13. **No file exceeds 200 lines.** Including doc comments. Enforced by `cargo xtask check-len` in
    CI, not by good intentions. See *Code structure* below for how tests stay near the code.
14. **Ports point inward; adapters plug in from outside.** Domain logic never names a concrete
    adapter. Every extension point is a trait in an inner crate, implemented in an outer one, and
    wired exactly once in `episteme-server`.
15. **Authorization is a mandatory filter, never a post-filter.** A principal's visibility
    predicate is ANDed into the query *before* the index runs. Searching first and discarding
    afterwards leaks the existence, count and ranking of rows the caller cannot see. Fail closed:
    absent policy means deny.
16. **The embedded UI is an ordinary client.** It authenticates and authorizes through the same
    path as gRPC. There is no localhost bypass, no privileged internal route, no "it's the admin
    console so it's fine."
17. **Points carry named vector fields, not one vector.** Each field has its own model, dim, metric,
    index and presence bitmap (ARCHITECTURE §4.1). Never assume a point has exactly one vector, and
    never assume a field is populated — check the presence bitmap.
18. **A named vector field declares its own query encoder.** Searching an image field with text must
    route to the joint model's *text tower*, not the collection's text embedder. Getting this wrong
    returns plausible garbage rather than an error.
19. **The database stores content references, not media.** URI + range + hash. Blobs stay outside;
    media decoding (ffmpeg, resampling, frame extraction) never enters the core crates.
20. **Never embed what must stay secret.** Sensitive spans are redacted *before* the embedder, not
    after. A vector computed from secret text leaks that text no matter what guards the payload,
    and no cryptography available to this project undoes that. Redaction is the control; see
    AGENT_START.md §10.2.
21. **Plugins never bypass policy.** A plugin runs as a principal with grants like any other caller.
    There is no plugin-privileged path, and a connector never receives `read_vector` by default.
22. **Index and distance extensions are compile-time, permanently.** A WASM boundary crossing per
    distance computation would dominate every query. "Bring your own search algorithm" is served by
    publishing episteme as a **crate** — depend on it, implement `VectorIndex`, build your binary.
23. **A source plugin is a `SourceReader`, not a parallel ingest path.** It emits the same record
    stream bulk import already consumes, inheriting jobs, checkpoints, resume and rejects. If plugin
    ingest needs new failure machinery, the design has drifted.
24. **Plugins compute; apps compose.** The app layer is declarative — a manifest and a DAG, never
    arbitrary code. The moment an app can execute logic, it is a plugin and the layer is pointless.
25. **"Vault" names a cryptographic guarantee, never an ACL.** A collection with an owner predicate
    is a *private collection*. Only a key-wrapped collection is a vault, and only a client-held key
    makes it *sealed*. Never let product language outrun the actual guarantee (ARCHITECTURE §7.1).
26. **Auto-vault classification is monotonic.** A classifier may only move content *into* a vault,
    never out. That direction is fail-secure, which is the only reason a probabilistic model is
    permitted anywhere near this boundary.
27. **A locked vault is reported, not silently skipped.** It sets `complete = false` and names the
    locked vault. A user must be able to tell "no results" from "no results you can currently see".
28. **Telemetry never emits a vector, a payload, or a vault name.** Logs land in systems with
    weaker access control than the database and are read by people granted nothing. A query vector
    in a log is `read_vector` for anyone with log access — and it can be inverted toward its source
    text. Emit shape (`dim=768`), never values. Enforced by
    `crates/episteme-index/tests/telemetry_leaks.rs`.
29. **Metric labels are bounded; spans carry the rest.** Segment ids, generations, job ids,
    principals and resource names are span fields, never metric labels — as labels they multiply
    time series without limit and take the monitoring system down. `fields::LABEL_SAFE` is the
    allowlist, and a test enforces it.
30. **Field and metric names are constants from `episteme-telemetry`.** A span keyed `collection`
    in one crate and `collection_name` in another cannot be joined, and nothing surfaces the
    mistake until someone tries to query the data.
31. **Library crates get facades only.** `tracing` and `metrics` compile to near-nothing with no
    subscriber installed. Exporters, config and any async runtime live behind the `subscriber`
    feature and are wired exactly once, in a composition root.
32. **Every public item carries a doc comment.** Struct, field, enum, variant,
    function, constant, associated item — all of them. Enforced by
    `#![deny(missing_docs)]` in every crate, so an undocumented item fails the
    build rather than the review.

    A comment must say what the item is *for*, or what breaks without it — never
    restate its own name. `/// The name.` above `fn name()` passes the compiler
    and helps nobody; `cargo xtask check-docs` catches those.

    On a field, say what the value means and what range or state is valid. On a
    function, say what it does and — where it matters more — what it refuses. On
    anything load-bearing, say why it is that way, because that is the part
    nobody can reconstruct from the code.
33. **A probabilistic classifier never gates a security boundary.** Regex, NER and schema-declared
    sensitivity are the enforcement layer. An LLM may *propose* labels for approval, never
    authorize on its own — a 98%-accurate detector leaks 2% silently and forever.
34. **One visibility predicate, every access path.** A row reached by graph traversal is checked by
    the same predicate as a row reached by top-k. Never write a second authorization path for the
    graph; two systems that must agree are how leaks happen.
35. **`PolicyEngine` returns a predicate, not a boolean.** `Decision { effect, row_predicate,
    field_mask }`. A boolean-returning port cannot express row-level visibility and cannot be
    retrofitted cheaply. This holds no matter which engine backs it — built-in, OPA/Rego, Cedar.
36. **Policy is resolved once per query, never per row.** The engine produces a `VisibilityContext`
    that compiles to a bitmap, cached on `(principal, collection, policy_version)`. Calling a
    policy engine inside a traversal or scan loop is a correctness-of-design failure, not a
    performance nit — an HNSW query visits thousands of nodes.
37. **The Google AIP linter is the only linter for `.proto` files.** Never add `buf lint`, and
    never configure a second opinion about the API surface. Two linters disagreeing get
    reconciled by suppressing one of them, which is exactly what rule 38 forbids — so the
    conflict is removed at the source instead.

    `buf format` and `buf breaking` are fine and expected: formatting is not an opinion about the
    API, and breaking-change detection is about wire compatibility rather than design.
38. **Never suppress the API linter.** No `(-- api-linter: ... --)` disables, no exclusions
    added to the lint config, no rule silenced to make a build green. If a lint fires, the API is
    wrong — change the API.

    CI runs the linter with `ignore-comment-disables: true`, so an in-proto suppression is not
    merely against policy: it has no effect. The rule is enforced rather than trusted.

    The linter is the only thing enforcing that a resource name means the same thing in every
    projection of the schema. A suppression is not a local exception; it is a permanent
    divergence between what the proto says and what the ecosystem assumes, and it surfaces much
    later as two systems disagreeing about identity.

    If a rule is genuinely inapplicable to this project, that is a conversation to have once, in
    the lint configuration, with the reason written down — never an inline suppression on one
    field.
39. **Generated protobuf code is committed, never built.** `cargo build` must need no
    protobuf toolchain. Regenerate with `cargo xtask gen-proto`; `cargo xtask check-proto` fails
    CI if the committed output drifts from the protos. Never hand-edit `src/generated/`.
40. **A shipped field number is permanent.** Never renumber, never reuse a tag, mark
    removals `reserved`. A segment written under an older schema still names its fields by
    number, so renumbering silently reinterprets stored data rather than failing.

---

## Workspace layout

```
episteme/
├─ Cargo.toml                 # workspace root
├─ protobuf/                  # .proto files — the API source of truth
├─ xtask/                     # dev tooling; owns the file-length + layering checks
├─ crates/
│  ├─ episteme-core/          # ontology: ids, domain types, errors, config schema
│  ├─ episteme-distance/      # SIMD distance kernels + runtime dispatch
│  ├─ episteme-storage/       # segments, WAL, manifest, mmap, quantization codecs
│  ├─ episteme-index/         # VectorIndex port; flat, hnsw, ivfpq adapters
│  ├─ episteme-query/         # filter evaluation, query planner
│  ├─ episteme-telemetry/     # span/metric vocabulary, redaction, subscriber wiring
│  ├─ episteme-policy/        # authz: principals, roles, grants, policy evaluation
│  ├─ episteme-io/            # bulk import/export: archive format, jobs, readers, rejects
│  ├─ episteme-embed/         # Embedder port, GGUF loader, candle adapter
│  ├─ episteme-embed-llama/   # OPTIONAL FFI adapter (feature = "llama")
│  ├─ episteme-graph/         # Plan A1.1 — property graph + traversal
│  ├─ episteme-proto/         # buf-generated, committed; no build script
│  ├─ episteme-ui/            # embedded web assets (rust-embed) + HTTP handlers
│  ├─ episteme-server/        # binary: composition root, gRPC services, observability
│  └─ episteme-client/        # Rust SDK
├─ sdk/{python,typescript}/   # generated clients
├─ ui/                        # UI source; built assets baked into episteme-ui
├─ benches/                   # criterion/divan benchmarks + recall harness
└─ docs/
```

Dependencies point **inward, toward `core`**. `core` depends on nothing in the workspace and
knows about no I/O. `server` is the composition root — the only place adapters are chosen and
wired. If you need an outward dependency, the abstraction is in the wrong crate: move the trait
inward, not the implementation outward.

---

## Code structure

The shape is **ports and adapters**, applied for practical benefit rather than doctrine. The goal
is that each segment of the system has an explicit vocabulary — its *ontology* — so that adding a
new capability later means adding a file, not editing ten.

**Three layers per crate:**

```
crates/episteme-index/src/
├─ lib.rs                 # re-exports only, ~20 lines
├─ domain/                # pure types + logic. No I/O, no tokio, no file handles.
│  ├─ mod.rs
│  ├─ candidate.rs
│  └─ params.rs
├─ ports/                 # the traits that define this segment's boundary
│  ├─ mod.rs
│  └─ vector_index.rs
└─ adapters/              # implementations, one per directory
   ├─ flat/
   ├─ hnsw/
   └─ ivfpq/
```

`domain` is testable with no setup. `ports` is the contract — changing one is a design decision.
`adapters` are replaceable and, ideally, boring.

**The ports.** These are the extension points; treat them as the system's real API:
`VectorStore`, `VectorIndex`, `BlockReader`, `Embedder`, `SourceReader`, `ArchiveWriter`,
`JobStore`, `PolicyEngine`, `GraphStore`. Adding an adapter must never require touching `core`.

### The 200-line rule

Every `.rs` file stays under 200 lines, doc comments included. Consequences worth planning for:

- **One public concept per file.** A trait and its implementations never share a file. An enum
  with substantial `impl` blocks gets its own file.
- **`mod.rs` is declarations and re-exports only.** If it holds logic, that logic wants its own file.
- **Tests move to a sibling file**, since they count toward the limit:

  ```rust
  // in foo.rs, last two lines
  #[cfg(test)]
  #[path = "foo_test.rs"]
  mod tests;
  ```

  This keeps tests adjacent in the directory listing without inflating the implementation file.
- **Split along conceptual seams, not line counts.** The failure mode of this rule is forty files
  each holding one function, where the reader can no longer follow a thought. If a file is over
  the limit and there is no meaningful seam, that is a signal the abstraction is wrong — fix the
  design rather than slicing the file at line 200.
- Generated code (`episteme-proto`) and vendored fixtures are exempt; the checker skips
  `OUT_DIR` and anything marked `@generated`.

---

## Commands

The workspace does not exist yet; these are the intended shapes. Keep them working as it lands.

```bash
cargo build --workspace                 # must succeed with zero external toolchains
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p episteme-server -- --config episteme.toml

cargo bench -p episteme-index           # latency
cargo run -p episteme-index --bin recall -- --dataset sift-1m   # recall@k vs flat

cargo build -p episteme-embed-llama --features llama            # opt-in FFI path
buf format --diff --exit-code            # buf lint is never used — see rule 37

cargo xtask check-len                   # fails on any .rs over 200 lines
cargo xtask check-docs                  # fails on an undocumented or empty doc comment
cargo xtask gen-proto                   # regenerate Rust from protobuf/ (needs buf)
cargo xtask check-proto                 # fails if the committed generated code has drifted
cargo xtask protodoc                    # regenerate protobuf/**/README.md
cargo xtask check-protodoc              # fails if the committed protobuf docs are stale
cargo xtask check-layers                # fails on an outward crate/module dependency
```

Run a single test: `cargo test -p episteme-storage segment::tests::seal_is_atomic`

---

## Conventions

**Errors.** `thiserror` for library crates, one error enum per crate. `anyhow` only in
`episteme-server` binaries and tests. Errors that cross gRPC map to explicit `tonic::Status`
codes in one place — never `.unwrap()` into a 500.

**Unsafe.** Allowed only in `episteme-distance` (SIMD intrinsics), `episteme-storage`
(mmap casts), and `episteme-embed-llama`. Every `unsafe` block carries a `// SAFETY:` comment
naming the invariant it relies on. Every other crate carries `#![forbid(unsafe_code)]`.

**Proto changes.** Additive only once a version ships. Never renumber a field, never reuse a tag.
`buf breaking` runs in CI. SDKs regenerate from proto — never hand-edit generated code.

**On-disk and archive formats.** Both are versioned and both are contracts. Changing the archive
layout is a bigger deal than changing the segment layout — segments are local and can be rewritten
by compaction, archives are out in the world. Round-trip tests (export → import → compare, including
edges) guard this; keep a golden archive fixture in CI.

**Testing.**
- Unit tests next to the code.
- `proptest` for anything that round-trips through bytes: codecs, segment serialization, WAL framing.
- Crash-consistency tests for the WAL and manifest — simulate a truncated write, assert recovery.
- Recall tests for indexes, with a committed ground-truth fixture.

**Async.** `tokio` in the server only. Storage and index crates are synchronous and
runtime-agnostic; that keeps them benchable and embeddable.

**Config.** One `episteme.toml`, deserialized via `serde` into typed structs in `episteme-core`.
Every field documented and defaulted. No stringly-typed config lookups scattered through the code.

---

## Gotchas that will bite

- **mmap and tail latency.** Page faults are invisible until memory pressure makes them the whole
  p99. Keep the read path behind a `BlockReader` trait so a direct-IO backend can replace mmap
  later without touching index code.
- **Cosine.** Normalize on ingest, then use dot product. Do not normalize per query at search time.
- **HNSW and deletes.** Deleting from an HNSW graph degrades it. Use tombstones + rebuild on
  compaction; do not attempt in-place graph deletion.
- **Filtered search is not "search then filter."** A selective filter with post-filtering returns
  too few results. This needs a planner — see `AGENT_START.md` §Filtering.
- **GGUF is not universal.** It covers the model architectures the loader implements, not every
  model on HuggingFace. Scope is encoder-style embedding models (bge, e5, gte, nomic, jina).
- **Jetson is CUDA-on-aarch64.** It is a cross-compilation and driver-version problem, not a code
  problem. Do not claim support without having run it on the device.
- **macOS gets no GPU inside a container.** Not Docker, not Apple's `container` — Apple GPUs have
  no IOMMU and `Hypervisor.framework` exposes no virtual GPU. On macOS the binary runs natively;
  containers are the Linux story. Never write a Metal code path that assumes a container runtime.
- **The desktop app is packaging, not architecture.** If logic ends up in the Tauri layer instead
  of the server, the boundary is wrong — the browser and the app must reach identical behaviour.
- **Never introduce cross-segment state.** A shared centroid table, a graph spanning segments, an
  ID map consulted across segments — each silently breaks scatter-gather and forecloses clustering
  (AGENT_START.md §14.3). IVF centroids are per-segment or replicated read-only, never mutable and
  shared. This costs nothing today and is expensive to unwind later.
- **Vectors never cross the wire as `repeated float`.** Protobuf encodes each element separately —
  768 varint ops per message on the hot path. Use `bytes` with raw little-endian f32 and cast.
- **`SearchResponse` carries `complete` / `shards_answered` / `shards_total` from day one.**
  Single-node sets `complete = true` unconditionally. Adding these later is a breaking change to
  the most-used message in the API (AGENT_START.md §15.6).
- **Standalone fans out to itself.** `Transport::InProcess` is a direct call, never a loopback RPC.
  One code path for one node and many is the only way the distributed path stays honest.
- **Bulk file ingest is server-side.** For large corpora the client sends a glob, not bytes.
  Streaming hundreds of GB through gRPC is the wrong shape; keep `SourceReader` between the job
  and the filesystem so object storage can slot in later.
- **Edges import in a second pass.** Nodes first, then edges resolved against the external→internal
  map. A dangling edge follows the configured policy — it never silently disappears.
- **Reject files are import input.** Keep the raw record intact in a reject, not just an error
  string, so a fixed reject file can be re-submitted directly.
- **Scores leak.** A caller who can only issue queries and read similarity scores can still probe
  out content. Return ranks or quantized scores to low-trust principals, and rate-limit.
- **Encrypted search is not on the table.** FHE is 10⁴–10⁶× too slow and destroys the index;
  order-preserving encryption is broken; a secret orthogonal transform is obfuscation, not
  encryption, and must never be described as the latter. Encryption at rest is the sanctioned use
  of cryptography here.

---

## Working style for this repo

- The user is architecting alongside you. For anything touching the on-disk format, the index
  trait, or the proto contract: **propose, get agreement, then implement.** These are expensive to
  reverse.
- Prefer landing a narrow vertical slice that runs end-to-end over a wide layer that runs nowhere.
- Benchmark before optimizing. This codebase will attract premature SIMD; resist it until a
  profile justifies it.
- When a phase from `AGENT_START.md` completes, update the status markers there in the same change.
