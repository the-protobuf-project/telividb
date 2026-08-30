# CLAUDE.md

Guidance for Claude Code when working in this repository.

> **New to this repo? Read [`AGENT_START.md`](./AGENT_START.md) first.** It holds the
> architecture, the phased roadmap, and the explicit list of what is and is not in scope.
> This file is the *working rules*; that file is the *plan*.

---

## What this is

**telividb** — a single-node, embedded-or-served **vector database** written in Rust, with a
gRPC API as the primary interface. Three things distinguish it from the field:

1. **Bring your own embedding model.** Models load from **GGUF** files, described by a config
   file. Inference runs on whatever hardware is present without recompiling the database.
   `candle` backs this today; ggml or ONNX may join it, because a model runtime sits *above* the
   engine and is swappable per field behind the `Inferencer` port (rule 42). Where no local
   runtime has a path for a model, the answer is "not yet," routed through the narrow
   `RemoteEmbedder` escape hatch.
2. **Bring your own search algorithm.** ANN indexes are pluggable behind a trait. Flat, HNSW
   (`instant-distance`) and IVF/IVF-PQ ship in-tree; their selection, partitioning and traversal
   logic is pure Rust over `&[f32]` and knows nothing about hardware. Only dense scoring reaches
   the tensor runtime, which is why an index survives a runtime change untouched (rule 46).
3. **Every embedding call, every plugin's compute step, and every query-time encode goes through
   one inference server.** It is core, not an adapter a plugin can route around — see invariants
   42–45.

---

## The five layers

The shape everything else hangs off. Each layer may use the one below it and must not reach
past it; a layer names an abstraction, never a backend.

| # | layer | what lives there | crates |
|---|---|---|---|
| 1 | **compute** | the tensor runtime and every `unsafe` line that talks to it | `telividb-compute` |
| 2 | **algorithms** | distance kernels, k-means, PQ codebooks, bounded selection | `telividb-distance` |
| 3 | **indexing** | flat, IVF, IVF-PQ, HNSW — composing layers 1 and 2 | `telividb-index` |
| 4 | **models** | embedding and scoring models; several runtimes allowed | `telividb-embed` |
| 5 | **apps** | the server, the SDK, plugins, examples | `telividb-server`, `telividb-client` |

**Layers 2, 3 and 5 must survive a layer-1 change untouched.** They operate on `&[f32]` and
plain Rust types; if an algorithm names a device, a backend or a tensor, it is in the wrong
layer. That property is what made replacing the compute runtime a contained change rather than
a rewrite, and it is worth protecting.

**Layers 2 and 3 reach layer 1 through a factory and an abstraction, never a raw handle.**
`Device::best()` picks hardware; a backend is obtained from it. No index calls a runtime
function directly.


Plan A is the vector store. Plan A1.1 layers a property graph over the same storage so the
system becomes a **graph + embedding database** for GraphRAG-style retrieval.

**Policy enforcement is not deferred.** `regorus` runs for real from the first vertical slice —
both at the query planner (rule 15) and at the inference-server boundary (rule 44). The first two
reference plugins built against this codebase are a voice-transcription slice and an OCEAN
personality-inference slice; the second exists specifically to force permission enforcement to be
real rather than assumed, since its output fields are sensitive-category data from the moment
they're written.

MCP has two directions in this project, and neither is a casual addition. **Emitting** an MCP
surface is generated from the descriptor set (`protoc-gen-mcp`) — never hand-built. **Consuming**
an external MCP server is an ordinary `SourceReader`-backed source plugin, same as any other
connector. **Do not hand-roll a bespoke MCP bridge outside those two paths** unless explicitly
asked — that is the thing this rule has always forbidden, not either generated/standard path.

---

## Non-negotiable invariants

Violating any of these is a bug, not a style preference.

1. **Everything is Rust except the tensor runtime.** ggml is C/C++ and is built with CMake;
   every other line in this workspace is Rust. That is a change from an earlier "100% Rust"
   promise, and it was made deliberately — the reasoning is in rule 42 and it is about hardware
   coverage, not convenience.

   **All FFI lives in one crate.** `telividb-compute` owns the ggml bindings, the build script
   and every `unsafe` block that talks to it. Every other crate keeps `#![forbid(unsafe_code)]`
   and sees a safe Rust API with methods on types — never a raw pointer, never a backend handle.
   That containment is what makes the trade acceptable: the FFI surface is one crate wide rather
   than workspace wide, and a reviewer knows exactly where to look.

   The other sanctioned native paths, unchanged: the optional `whisper.cpp` transcription
   backend (quarantined for the voice slice) and, if it is ever wired in, the optional FAISS
   index behind a non-default feature (rule 46).

   **Three C paths, recorded rather than hidden.** The mandated telemetry stack depends on MCAP,
   which depends on lz4, which builds with `cc`. It cannot be switched off from our side, because
   Cargo unifies features as a union.

   The second is **TLS, and it was a decision rather than an inheritance.** Installing a model
   means fetching it over HTTPS, and every TLS stack in Rust bottoms out in C or assembly for its
   crypto — `rustls` on `aws-lc-rs` (C and CMake) or on `ring` (C and assembly); `native-tls` on
   OpenSSL. There is no production-grade pure-Rust option to pick instead. So `aws-lc-sys` reaches
   the default build through `telividb-server`, which turns on `telividb-models/network`.

   Kept as narrow as it can be: `telividb-models` is default-free, so anything embedding the
   catalog without downloading takes none of it, and `telividb-providers` links no HTTP client at
   all — it stores keys and names providers; the window makes the calls. The cost is one crate, in
   the binary that already needs a network.

   The third is **`ring`, and it appears only in `app/`.** Answering happens in the window, and a
   webview enforces CORS against every provider's origin — so the desktop build reaches a remote
   model through `tauri-plugin-http`, which performs the request in Rust where CORS does not apply.
   That plugin pins `reqwest 0.12`, whose every `rustls-*` feature routes to `ring`, and Cargo's
   feature unification then compiles `ring` alongside the `aws-lc-rs` already there. Two crypto
   backends in one binary is a genuine cost and it was weighed: the only alternative the plugin
   offers is `native-tls`, which on Linux means linking the system OpenSSL — trading a contained,
   Cargo-built C crate for an external one, on the target that ships as a daemon.

   **This one is also why the guard now sweeps `app/`.** That workspace is excluded from the
   repository's and was never checked, so `ring` entered the binary that actually ships while the
   `no-native-deps` job stayed green. The job runs over both workspaces now, and Tauri's own
   `embed-resource` and `objc2-exception-helper` are allowlisted there for the same reason ggml is
   here — expected, named, and failing the build if anything joins them.

   There used to be a second: `candle-core` pulled `tokenizers`, which pulled Oniguruma
   (`onig_sys`). Removing candle put that choice back in reach — `tokenizers` now takes
   `fancy-regex`, which is pure Rust — so the carve-out was **deleted rather than kept**. The
   workspace's native surface is smaller than it was before ggml arrived, not larger.

   The `no C toolchain` CI job's original premise is gone. What replaces it is narrower and still
   worth enforcing: **no *unexpected* native dependency.** ggml, CMake, the MCAP chain and the TLS
   chain above are expected; anything else fails the build and is a decision to record here,
   never a quiet addition to the allowlist. Note the order that makes this rule work — the
   paragraph above was written *because* the guard failed, not after quietly widening it.

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
7. **Vectorized scoring belongs to layer one, not to `telividb-distance`.** This replaces an
   earlier rule that asked for hand-written AVX2/AVX-512/NEON kernels with runtime dispatch in
   the distance crate. The reason is measured, on an M3 Max over a million 128-dimension rows:

   | path | per query | vs scalar |
   |---|---|---|
   | `FlatIndex` — scalar Rust | 42.403 ms | — |
   | ggml CPU backend, one query | 6.693 ms | **6.3x** |
   | ggml CPU backend, batch of 32 | 2.982 ms | **14.2x** |

   ggml's CPU backend already carries tuned kernels for exactly this operation, so a hand-written
   set would be reimplementing them worse — and it would put hardware knowledge in a layer whose
   whole purpose is not having any (rule 46). Bulk scoring on the host is
   `GpuFlatIndex::build_on(store, DeviceKind::Cpu)`, which is layer one on the CPU backend, not a
   different code path.

   **`telividb-distance` keeps its scalar kernels, and they stay scalar.** They are the
   correctness reference every recall number is measured against (rule 8), and a reference that
   is itself optimized is no longer a reference. What the crate owns beyond them is the branchy,
   scattered work rule 46 keeps on the host anyway: k-means, PQ codebook training, ADC tables.

   **ggml is pinned to a tagged release, never to master.** A tensor runtime moving under the
   engine changes numerical output, so a recall or cosine figure measured against an untagged
   commit is one nobody can reproduce — including this repository a week later. The submodule
   records a commit either way; the rule is that the commit is one upstream *named*. Enforced
   by a step in the `invariants` job, because the pointer had already drifted one commit past
   `v0.22.0` before anyone looked. Bumping it is deliberate: check out the new tag, commit the
   pointer, re-run recall. Never `git submodule update --remote`.

   **`target-cpu=native` is still never required to run**, and that now has to be enforced rather
   than assumed: ggml's own default compiles with `-march=native`, which faults with SIGILL on
   any older CPU. `telividb-compute` states the choice explicitly and exposes a `portable`
   feature for anything shipped to a machine that did not compile it.
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
    degrades recall silently, which is the worst failure mode available. A model identity is
    always a GGUF hash under this codebase — there is no `format` alternative to check.
13. **No file exceeds 200 lines.** Including doc comments. Enforced by `cargo xtask check-len` in
    CI, not by good intentions. See *Code structure* below for how tests stay near the code.
14. **Ports point inward; adapters plug in from outside.** Domain logic never names a concrete
    adapter. Every extension point is a trait in an inner crate, implemented in an outer one, and
    wired exactly once in `telividb-server`.
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
20. **Never embed what must stay secret.** Sensitive spans are redacted *before* they reach the
    inference server, not after. A vector computed from secret text leaks that text no matter what
    guards the payload, and no cryptography available to this project undoes that. Redaction is
    the control; see AGENT_START.md §10.2.
21. **Plugins never bypass policy.** A plugin runs as a principal with grants like any other caller.
    There is no plugin-privileged path, and a connector never receives `read_vector` by default.
22. **Index and distance extensions are compile-time, permanently.** A WASM boundary crossing per
    distance computation would dominate every query. "Bring your own search algorithm" is served by
    publishing telividb as a **crate** — depend on it, implement `VectorIndex`, build your binary.
23. **A source plugin is a `SourceReader`, not a parallel ingest path.** It emits the same record
    stream bulk import already consumes, inheriting jobs, checkpoints, resume and rejects. If plugin
    ingest needs new failure machinery, the design has drifted. This applies identically whether the
    source is a bespoke API, a filesystem, or an external MCP server (see "What this is").
24. **Plugins compute; apps compose.** The app layer is declarative — a manifest and a DAG, never
    arbitrary code. The moment an app can execute logic, it is a plugin and the layer is pointless.
25. **"Vault" names a cryptographic guarantee, never an ACL.** A collection with an owner predicate
    is a *private collection*. Only a key-wrapped collection is a vault, and only a client-held key
    makes it *sealed*. Never let product language outrun the actual guarantee (ARCHITECTURE §8.1).
26. **Auto-vault classification is monotonic.** A classifier may only move content *into* a vault,
    never out. That direction is fail-secure, which is the only reason a probabilistic model is
    permitted anywhere near this boundary. Any newly declared sensitive-category field (e.g. an
    inferred personality/trait score) defaults to vault-eligible at schema-authoring time, not
    after the fact.
27. **A locked vault is reported, not silently skipped.** It sets `complete = false` and names the
    locked vault. A user must be able to tell "no results" from "no results you can currently see".
28. **Telemetry never emits a vector, a payload, or a vault name.** Logs land in systems with
    weaker access control than the database and are read by people granted nothing. A query vector
    in a log is `read_vector` for anyone with log access — and it can be inverted toward its source
    text. Emit shape (`dim=768`), never values. Enforced by
    `crates/telividb-index/tests/telemetry_leaks.rs`.
29. **Metric labels are bounded; spans carry the rest.** Segment ids, generations, job ids,
    principals and resource names are span fields, never metric labels — as labels they multiply
    time series without limit and take the monitoring system down. `fields::LABEL_SAFE` is the
    allowlist, and a test enforces it.
30. **Field and metric names are constants from `telividb-telemetry`.** A span keyed `collection`
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
    authorize on its own — a 98%-accurate detector leaks 2% silently and forever. This applies to
    the OCEAN reference plugin's output exactly as it applies to redaction: the model proposes
    trait scores, it never grants itself a permission scope.
34. **One visibility predicate, every access path.** A row reached by graph traversal is checked by
    the same predicate as a row reached by top-k. Never write a second authorization path for the
    graph; two systems that must agree are how leaks happen.
35. **`PolicyEngine` returns a predicate, not a boolean.** `Decision { effect, row_predicate,
    field_mask }`. A boolean-returning port cannot express row-level visibility and cannot be
    retrofitted cheaply. The shipped adapter is `regorus`; this holds regardless of which engine
    backs the trait.
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
41. **Telemetry goes through `telemetry-rs`, always.** The ecosystem's stack —
    `the-protobuf-project/telemetry` — is the pipeline for logging, tracing, metrics and MCAP
    recording. Never hand-roll a `tracing-subscriber` stack, never add a second exporter, never
    reach for `tracing-appender` or `metrics-exporter-prometheus` directly, and never put a
    facade in front of it.

    **There is no facade.** Every crate that emits depends on `telemetry` directly and logs
    through `logger::info!` / `logger::debug!` / `logger::warn!` / `logger::error!`. The
    `tracing` and `metrics` crates are not dependencies of this workspace and must not be
    reintroduced: the stack installs a `tracing` subscriber only when an OTLP tracer exists, so a
    `tracing::` call site is silent in the default configuration, and the stack installs no
    `metrics` recorder at all, so a `metrics::` call site is silent in *every* configuration.
    Both failures are invisible at the call site, which is how the instrumentation came to be
    disconnected from the pipeline while looking correct.

    This is safe in a library crate because emission is a no-op until a composition root builds
    the pipeline: `logger::` checks a `OnceLock` and returns, and a `Meter` defaults to disabled.
    `telividb-storage` and `telividb-index` stay synchronous, benchable and embeddable.

    **Metrics need a handle.** `telemetry::Metrics` records through `&mut self` and the stack
    exposes no global for it. Since every telividb call site that records sits behind `&self`,
    the recorder is shared as an `telividb_telemetry::Meter` — construct one from the pipeline in
    the composition root and pass it in (`with_meter`). A default `Meter` records nothing, which
    is what every test and benchmark uses. Note the stack's metrics take **no attributes**: a
    dimension like index kind travels on the log record, not as a metric label.

    Use it the way its own examples do: `Telemetry::new().with_service(name, version)
    .environment(Environment::…).build()` — not `Telemetry::builder(...)`.

    **Verbosity is a log level, and it lives in `telemetry.toml`.** `[logging] level` (1=error,
    2=info, 3=debug) sets it, `[logging.modules.<name>]` overrides per module, and
    `.with_log_level()` overrides both from code. `--log-level` maps onto it and is left unset by
    default *so that the file decides* — a flag that always won would make that section dead.
    `Environment` is a resource attribute telling a collector which deployment a record came
    from; it is not the verbosity control.

    `telemetry.toml` is therefore load-bearing, and its `[telemetry] enabled` is the master
    switch for the OTLP pipeline. The file is discovered relative to the process working
    directory, so a deployed binary sets `TELEMETRY_CONFIG_PATH` rather than assuming its CWD.

    Four consequences worth knowing rather than rediscovering: building the pipeline needs a
    tokio runtime, so anything testable without one belongs in a free function; `logger::info!`
    returns a builder that emits on drop, so a match arm needs a block; `flush()` reports an
    error when no collector is configured, because the meter has nothing to force-flush, so a
    clean shutdown must not treat that as fatal; and the stack installs a *global* logger, so a
    test binary gets exactly one `install` and must share its result rather than installing per
    test.
42. **One tensor runtime under the engine; several are fine above it.** The distinction is the
    rule, and it replaces an earlier one that named candle as the only runtime and forbade ggml
    by name.

    **Layer one — compute — is ggml, and only ggml.** `telividb-compute` is the single crate that
    binds a runtime, and everything that scores vectors goes through it. The reason is hardware
    coverage and it is checkable rather than aesthetic: candle's `Device` enum has exactly three
    variants — `Cpu`, `Cuda`, `Metal` — so Intel and AMD GPUs are unreachable from it at any
    effort, and its `mkl` feature is Intel's *CPU* math library, not Intel GPU. ggml carries
    CUDA, Metal, Vulkan, HIP, SYCL, OpenCL and more behind one backend interface. The earlier
    rule's actual principle — *one runtime, one set of hardware backends to reason about* — is
    preserved; what changed is which runtime satisfies it.

    ggml is **graph-based, not eager**. Tensors are declared into a context, wired into a graph,
    and computed in one call, so there is no free-floating `a.matmul(b)` returning a value. The
    API above it therefore expresses **whole jobs** — "score this batch against this corpus" —
    not individual operations. That is also the only shape that keeps a device busy, so the
    constraint and the performance advice point the same way.

    **Layer four — models — may have several runtimes, and that is deliberate.** ggml backs the
    embedding path today, and it is the only one: the encoder is built from `telividb-compute`
    graph operations, so a GGUF model runs on the same runtime the index scores on, with one
    quantization implementation and one loader rather than two.

    ONNX through `ort` is the standing provision, for architectures no GGUF loader reaches. It
    would be a sibling adapter behind the same `Inferencer` port and would touch no layer below.
    The constraint that mattered belongs to layer one, not here.

    **candle was removed, not deprecated.** It was layer four's runtime while it was also layer
    one's; once layer one moved to ggml, keeping it meant two tensor runtimes resident, two GGUF
    loaders and two quantization paths for one model family. The ggml encoder reproduces the
    candle one's output on the same model to a cosine of 1.000000 — recorded as a committed
    fixture in `telividb-embed/tests/fixtures/`, so the guarantee outlived the implementation it
    was checked against.

    The `RemoteEmbedder` escape hatch stands unchanged: a declared network call with the same
    provenance tracking as a local model (rule 12), for hardware no in-process runtime reaches.

43. **No plugin loads a model file directly.** A `transform`/`rerank`/`embedder`-kind plugin
    manifest declares a pinned GGUF reference (`[model]` block: path, sha256) and calls the
    inference server with it; the plugin process itself never opens a `.gguf` file or touches a
    GPU handle. This is what makes plugin sandboxing (rule 21) actually hold for compute
    plugins — a plugin that has no model-loading code cannot exceed its declared reference no
    matter what it's compromised into doing. `kind = "source"` plugins are exempt: they move
    records, they do not compute, so they carry no `[model]` block.
44. **The inference server checks policy before it runs a model, not only before results return.**
    A call into the inference server that would process data behind a `vector.<field>` permission
    scope is evaluated by `regorus` first (same `PolicyEngine` port as rule 35, a second call
    site). This is a stronger guarantee than the query-planner check alone: it stops a plugin from
    ever producing a vector or a score for data it wasn't granted, rather than only hiding the
    output afterward.
45. **The inference server holds multiple GGUF models resident at once; nothing swaps per call.**
    Do not write a code path that assumes "load model, run, unload" — that defeats the batching
    that makes GPU inference worth having in-process, and the OCEAN reference plugin alone needs
    an embedder and a scoring model live simultaneously. VRAM budgeting across resident models is
    an open design question (AGENT_START.md / ARCHITECTURE §15 Gap 22) — flag it, don't silently
    assume unlimited headroom.
46. **The device scores; the host decides.** Which operations run where is settled by
    measurement, not preference, and the measurements are on an M3 Max over SIFT-1M:

    | operation | runs on | why |
    |---|---|---|
    | exhaustive scoring | **device** | one contiguous matmul — 2.17 ms against 47 ms on the host |
    | batched scoring | **device** | the corpus is read once for the whole batch — 5.5x per query at 32 |
    | top-k selection | **host** | a bounded heap at 0.27 ms; a device sort cannot address a million columns |
    | ADC code lookup | **host** | a scattered gather with almost no arithmetic — 25x slower on a device |
    | graph traversal | **host** | each hop depends on the last |

    The governing quantity is **arithmetic intensity crossed with independence**. A device wins
    when the same bytes feed many independent operations; the host wins when work is dependent,
    branchy, or a low-arithmetic scattered gather. Every row above follows from that, and so does
    the counter-intuitive result below.

    **Do not partition on the device.** IVF on the GPU measured *slower than not partitioning* —
    2.27 ms at 1.6% probed against 2.17 ms for scanning everything — because gathering a subset
    costs more than scoring the whole corpus. IVF and PQ exist to avoid work a device does not
    find expensive. They stay on the host, where they are also what a corpus too large for device
    memory needs.

    **HNSW is not ported to a device and should not be.** Traversal is sequential pointer-chasing,
    the access pattern a GPU is worst at; FAISS ships no GPU HNSW for the same reason.

    **A device that quietly fell back to the host passes every correctness test while delivering
    none of the speed**, so the selected backend is emitted on every build and search
    (`fields::DEVICE`) rather than left to be inferred. Results must be identical on every
    backend — only the speed may differ.

    **The device corpus is rebuilt on load, never persisted.** It is exactly derivable from the
    store, and rebuilding a million rows measures at 0.14 s against the 512 MB a serialized copy
    would occupy — so a file would trade real disk for no recovery time. Nothing under
    `adapters/gpu/` writes one, and rule 4 has nothing to version there because there is no
    on-disk structure. This is a decision, not an omission: an earlier revision *did* carry a
    GGUF read/write path for it, reached only by its own tests, and removing it is what made the
    ggml migration a smaller change than the candle version it replaced. HNSW is the opposite
    case and still persists — its graph costs 697 s to build.

    **A `ggml` backend is not `Sync`, and the wrapper does not pretend otherwise.** One backend
    holds one command queue, so concurrent compute submissions race. `telividb-compute` marks its
    types `Send` and not `Sync`, and the index holds its corpus behind a mutex taken for the
    device call alone — released before selection, so the host-bound half of a search still runs
    concurrently. This costs nothing real: a GPU executes submitted work serially anyway, and
    candle held an equivalent internal mutex for the same reason. Do not "fix" this with an
    `unsafe impl Sync`.

    A C++-backed *index* remains opt-in and never default: FAISS, if ever wired in, lives behind
    a non-default `faiss-index` feature. That is separate from ggml, which is the compute runtime
    beneath every index rather than an index of its own.

47. **The in-memory graph is rehydrated, not its own persisted format.** `telividb-graph` wraps
    `petgraph`, built in memory from `telividb-storage`'s edge records on collection load. This is
    a stated v1 capacity ceiling — very large, dense-edge collections become RAM-bound before the
    vector side does — not an oversight to "just fix" with a quick patch. If a workload needs
    more, that is a persisted-CSR design conversation (propose, get agreement, then implement —
    see *Working style*), not a silent local optimization.
48. **A point's time-to-live expires through the same tombstone-then-compaction path as an
    explicit delete, never a second mechanism.** TTL (`telividb.v1.ttl`) is a trigger, not new
    machinery: on expiry, tombstone immediately (so the query planner excludes it right away) and
    let the existing forced-compaction purge path (rule 9's neighbor, hard delete) physically
    remove it later.
49. **A locked vault, a policy denial, and an inference-server refusal are all reported, never
    silently absorbed into "no results."** Rule 27 already covers the locked-vault case; the same
    posture extends to the inference-server's pre-check in rule 44 — a caller whose request was
    refused by policy at the inference boundary gets a distinguishable error, not a quietly empty
    result.
50. **`protobuf/annotations/` never contains a business resource; `protobuf/schemas/` never
    defines a facet option.** The `telividb.v1` vocabulary (`point`, `vector`, `edge`, `span`,
    `content_ref`, `redact`, `ttl`) lives exclusively under `protobuf/annotations/telividb/v1/`.
    Every collection schema — voice, OCEAN, any plugin's own `.proto` — lives under
    `protobuf/schemas/` (or the plugin's own repo) and *imports* the annotations, never the other
    way. A message that is both a `google.api.resource` and a facet-option definition in the same
    file is a sign the split has been violated; split the file instead of arguing the exception.
51. **A crate under `crates/domain/` or `crates/adapters/` must build and pass its tests with only
    its declared `Cargo.toml` dependencies — never an implicit path back into `telividb-server` or
    a sibling crate's private internals.** `cargo xtask check-layers` enforces this in CI. This is
    what makes "publish `telividb-storage` on its own" a real option rather than an aspiration —
    the crate groupings in *Workspace layout* exist specifically so this boundary has an obvious
    place to be checked.

---

## Workspace layout

**Crates marked `PLANNED` do not exist yet.** This is the intended shape, not a
description of the tree — four of the sixteen below are unbuilt. `telividb-policy`
is the one that matters most: six invariants describe authorization that nothing
can currently enforce.

```
telividb/
├─ Cargo.toml                 # workspace root
├─ protobuf/
│  ├─ annotations/            # the FACET VOCABULARY ONLY — telividb.v1 — never a business schema.
│  │  └─ telividb/v1/         # point.proto, vector.proto, edge.proto, span.proto,
│  │                          # content_ref.proto, redact.proto, ttl.proto (ARCHITECTURE §2.2, §18)
│  ├─ schemas/                # collection schemas that USE the annotations — one package per domain
│  │  ├─ voice/v1/            # voice.proto — the voice reference slice (ARCHITECTURE §16.2)
│  │  └─ ocean/v1/            # ocean.proto — the OCEAN reference slice (ARCHITECTURE §16b)
│  ├─ buf.yaml
│  └─ buf.gen.yaml
├─ xtask/                     # dev tooling; owns the file-length + layering checks
├─ crates/
│  ├─ domain/                 # pure business logic. No I/O, no tokio, no file handles, no adapters.
│  │  ├─ telividb-core/       # ontology: ids, domain types, errors, config schema
│  │  ├─ telividb-query/      # query planner: the seed→expand graph join.
│  │  │                       # Filter evaluation still to come.
│  │  └─ telividb-graph/      # Plan A1.1 — petgraph-backed property graph + traversal (rule 47)
│  ├─ adapters/                # replaceable implementations of a domain port. I/O-shaped, boring.
│  │  ├─ telividb-storage/    # segments, WAL, manifest, mmap, redb metadata, quantization codecs
│  │  ├─ telividb-index/      # flat + instant-distance HNSW + ivfpq
│  │  │                       # (FAISS, if wired at all, behind a non-default feature — rule 46)
│  │  ├─ telividb-distance/   # SIMD distance kernels + runtime dispatch
│  │  ├─ telividb-embed/      # inference server: Inferencer port, GGUF loader, ONE adapter (candle)
│  │  │                       # GPU-resident, multi-model, batched — the sole call path for
│  │  │                       # ingest embedding, query_encoder, and every plugin's compute step
│  │  ├─ telividb-embed-llama/# PLANNED — optional FFI adapter for whisper.cpp (feature="llama")
│  │  ├─ telividb-policy/     # PLANNED — authz: principals, roles, grants, regorus
│  │  │                       # NOTHING ENFORCES RULES 15/21/34/35/36/44 UNTIL THIS EXISTS
│  │  └─ telividb-io/         # PLANNED — bulk import/export: archives, jobs, rejects
│  ├─ platform/                # cross-cutting concerns no single domain/adapter crate owns
│  │  ├─ telividb-compute/    # LAYER 1: ggml, vendored as a submodule and built here.
│  │  │                       # The only crate with FFI. Everything else sees a safe,
│  │  │                       # method-based API and keeps forbid(unsafe_code).
│  │  ├─ telividb-telemetry/  # span/metric vocabulary, redaction, subscriber wiring
│  │  ├─ telividb-proto/      # buf-generated from protobuf/, committed; no build script
│  │  └─ telividb-ui/         # PLANNED — embedded web assets + declarative panel handlers
│  ├─ sdk/                     # client libraries. No engine, no storage — the wire protocol only.
│  │  └─ telividb-client/     # Rust SDK: a gRPC client for telividb-server
│  └─ bin/                     # composition roots — the only place adapters get chosen and wired
│     └─ telividb-server/     # binary: gRPC services, inference server, observability
├─ sdk/{python,typescript}/   # generated clients
├─ app/                       # the desktop app, and its own bun workspace
│  ├─ packages/               # TypeScript packages — `packages/*` mirrors
│  │  └─ answer/              # `crates/*`: a piece with no Svelte in it,
│  │                          # buildable and testable on its own.
│  └─ src/                    # the SvelteKit app that consumes them
├─ ui/                        # UI source; built assets baked into telividb-ui
├─ benches/                   # criterion/divan benchmarks + recall harness
└─ docs/
```

**Plugins live in a separate repo, `telividb-plugins`, not under this tree.** That repo is its own
small Cargo workspace — `voice/` and `ocean/` today, any future first-party plugin alongside them
— and it depends on this repo only through what's actually public: the published
`protobuf/annotations/telividb/v1` module and the `SourceReader`/manifest contract (ARCHITECTURE
§10.2, §10.5), never a path back into `crates/`. That boundary is the point: a plugin author with
no access to this repo at all must be able to build the same thing `telividb-plugin-ocean` is,
using only what's published. If a change to a plugin ever requires touching a crate under
`crates/`, the plugin/core boundary has leaked and the fix belongs in the annotation vocabulary or
the `SourceReader` port, not in a special case for that plugin.

**Annotations are never mixed with business schemas.** `protobuf/annotations/telividb/v1/` holds
only the facet vocabulary itself (the `point`, `vector`, `edge`, `span`, `content_ref`, `redact`,
and `ttl` options referenced throughout ARCHITECTURE.md §2.2) — it defines *how* to annotate a
schema, and it must never contain a message that is itself a collection resource. Every actual
domain schema — the voice slice, the OCEAN slice, and any future plugin's `.proto` — lives under
`protobuf/schemas/<domain>/v1/`, imports the annotations package, and never the reverse. This
answers ARCHITECTURE §18's open question ("where does `telividb.v1` live, exactly?") for this
codebase: in this repo, under `protobuf/annotations/`, published to buf from there, kept
importable by a plugin's own out-of-tree `.proto` without pulling in any business schema.

**The `crates/` grouping mirrors the domain/ports/adapters split (see *Code structure* below) one
level up.** `domain/` crates depend on nothing outward and contain the logic that would still be
correct on a whiteboard with no database attached; `adapters/` crates are the replaceable,
I/O-shaped implementations of the ports `domain/` defines; `platform/` is the handful of
cross-cutting concerns (telemetry, generated proto code, the embedded UI) that don't belong to one
domain concept; `sdk/` holds client libraries, which speak the wire protocol and own no engine at
all; `bin/` is where everything gets wired together and nowhere else.

**`sdk/` is separate from `bin/` because a client is a library, not a binary.** Lumping them
together made `telividb-client` — which produces no executable — live in a directory named for
executables, and that misnomer is the kind that quietly justifies putting engine code in a client
later. The SDK depends on `telividb-proto` and nothing else; if it ever needs `telividb-storage` or
`telividb-index`, the boundary has been crossed and inference or search has leaked client-side,
which rules 42–45 exist to prevent. This is not
cosmetic directory tidiness — it is what makes the crates under `domain/` and `adapters/`
independently publishable.

**Every crate under `domain/` and `adapters/` is designed to be published standalone**, under the
`telividb` GitHub org, for reuse outside this server binary — `telividb-storage` or
`telividb-policy` should be usable by a project that wants a segment-based mmap store or a
`regorus`-backed row-visibility engine and nothing else from this codebase. This is why the
ports-and-adapters discipline (rule 14, and *Code structure* below) is enforced rather than
aspirational: a crate that quietly depends on `telividb-server` internals cannot be published
alone, so that dependency direction is checked in CI (`cargo xtask check-layers`), not just
documented.

Dependencies point **inward, toward `core`**. `core` depends on nothing in the workspace and
knows about no I/O. `server` is the composition root — the only place adapters are chosen and
wired. If you need an outward dependency, the abstraction is in the wrong crate: move the trait
inward, not the implementation outward.

---

## Two deployments, one engine

The same engine ships two ways, and the difference is **how long it lives** —
not what it does. Nothing below is a fork in behaviour; a question answered on
one is answered identically on the other.

| | macOS | Linux |
|---|---|---|
| shape | a desktop app | a daemon, the way `ollama` is one |
| engine lifetime | the window's | the machine's |
| transport | **Tauri IPC** | **gRPC-web** |

**The transport is chosen at runtime, in one place.** `resolveClient()` in
`app/src/lib/api/index.ts` returns the IPC adapter when `__TAURI_INTERNALS__` is
present on `window` and the gRPC-web adapter otherwise. Every panel is written
against the `TelividbClient` port and knows neither. Adding a third transport is
a third adapter and a line in that function.

**IPC first, gRPC-web second.** The desktop app is where this is being built and
used, so its transport is the one that has to work; the browser adapter is a
declared stub whose methods fail with a sentence naming what is missing. That
order is deliberate and the stub is honest — it is not a silent fallback.

**TypeScript is packaged the way Rust is.** `app/packages/*` is a bun workspace
mirroring `crates/*`, and the test is the same one rule 51 applies to a crate: a
package builds and passes its tests from its own `package.json`, with no path
back into the app. `@telividb/answer` is the first — no Svelte, no Tauri beyond an
optional peer — which is what lets the browser build that serves the Linux daemon
consume the identical code, and what gives the prompt and the guard somewhere to
be tested without a SvelteKit harness.

**Answering is in the window, and there is no sidecar.** The provider SDKs worth
using are TypeScript, and the window is already a TypeScript runtime — so it calls
them itself. There was briefly a Bun child process here on the reasoning that
"TypeScript SDKs" meant "Rust must call TypeScript"; that is circular, since it
ships a second JS runtime to serve the Rust process that serves the first one. It
was removed before anything depended on it. **Do not reintroduce a JS sidecar**
without a reason that survives the question *why can the window not do this?*

The engine's part is the half the window must not hold: `telividb-providers` keeps
the keys in the OS keychain and owns the provider table. Nothing in it links an
HTTP client.

**The cost of that choice, stated rather than buried.** The key is handed to the
window over IPC and lives in webview memory for the duration of a call, and the
"protected content stays local" check runs in TypeScript beside the call, where a
compromised or modified frontend can skip it. So the vault guarantee is currently
an intention, not a property. The fix is not a proxy: it is that a search declares
whether its passages are bound for a remote model, and the engine declines to
*return* protected passages when they are — enforcement on retrieval, which no
client can bypass because it never receives the content. `may_answer` in
`telividb-providers` is the server-side half, written and deliberately unwired
until that lands. Until then, do not describe a vault as enforced.

## Code structure

The shape is **ports and adapters**, applied for practical benefit rather than doctrine. The goal
is that each segment of the system has an explicit vocabulary — its *ontology* — so that adding a
new capability later means adding a file, not editing ten.

**Three layers per crate:**

```
crates/telividb-index/src/
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
   ├─ hnsw/                # instant-distance
   └─ ivfpq/
```

`domain` is testable with no setup. `ports` is the contract — changing one is a design decision.
`adapters` are replaceable and, ideally, boring.

**The ports.** These are the extension points; treat them as the system's real API:
`VectorStore`, `VectorIndex`, `BlockReader`, `Inferencer`, `SourceReader`, `ArchiveWriter`,
`JobStore`, `PolicyEngine`, `GraphStore`. Adding an adapter must never require touching `core`.
Note `Inferencer` replaced the older `Embedder` naming to make clear it is one port serving
ingest, query-time encoding, and plugin compute alike (rules 42–45) — it is not "the embedding
crate's trait," it is the single compute boundary for the whole system.

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
- Generated code (`telividb-proto`) and vendored fixtures are exempt; the checker skips
  `OUT_DIR` and anything marked `@generated`.

---

## Commands

The workspace does not exist yet; these are the intended shapes. Keep them working as it lands.

```bash
git submodule update --init --recursive   # ggml lives in telividb-compute/vendor
                                         # NEVER --remote: that walks to master's tip
cargo build --workspace                 # needs CMake for ggml (invariant 1); the
                                         # submodule above must be initialised first
cargo test  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

cargo run -p telividb-server -- --config telividb.toml

cargo bench -p telividb-index           # latency
cargo run -p telividb-index --bin recall -- --dataset sift-1m   # recall@k vs flat

# Compute backends. Metal is automatic on macOS; the rest are opt-in because
# each needs its own SDK at build time.
cargo build -p telividb-compute --features cuda                 # NVIDIA
cargo build -p telividb-compute --features hip                  # AMD
cargo build -p telividb-compute --features vulkan               # cross-vendor
cargo build -p telividb-compute --features sycl                 # Intel GPU

cargo build -p telividb-embed-llama --features llama            # opt-in FFI: whisper.cpp only
cargo build -p telividb-index --features faiss-index             # opt-in FFI: FAISS, never default
buf format --diff --exit-code            # buf lint is never used — see rule 37

cargo xtask check-len                   # fails on any .rs over 200 lines
cargo xtask check-docs                  # fails on an undocumented `pub` item or an empty doc comment
cargo xtask gen-proto                   # regenerate Rust from protobuf/ (needs buf)
cargo xtask check-proto                 # fails if the committed generated code has drifted
cargo xtask protodoc                    # regenerate protobuf/**/README.md
cargo xtask check-protodoc              # fails if the committed protobuf docs are stale
cargo xtask check-layers                # fails on an outward crate/module dependency
```

Run a single test: `cargo test -p telividb-storage segment::tests::seal_is_atomic`

---

## Conventions

**Errors.** `thiserror` for library crates, one error enum per crate. `anyhow` only in
`telividb-server` binaries and tests. Errors that cross gRPC map to explicit `tonic::Status`
codes in one place — never `.unwrap()` into a 500.

**Unsafe.** Allowed in `telividb-compute` (the ggml FFI — this is where it is expected to
live), `telividb-distance` (SIMD intrinsics), `telividb-storage` (mmap casts),
`telividb-embed-llama`, and, if the optional FAISS feature is built, `telividb-index`'s FAISS
adapter module specifically. Every `unsafe` block carries a `// SAFETY:` comment naming the
invariant it relies on. **Every other crate carries `#![forbid(unsafe_code)]`**, and that is the
containment rule 1 depends on — an `unsafe` block appearing outside this list is a design
failure, not a local exception.

**Methods, not free functions.** Behaviour hangs off a type: `impl` blocks with receiver
methods, so a reader finds an operation by looking at what it operates on. `Device::best()`,
`backend.name()`, `codebook.encode(v)` — not `best_device()`, `backend_name(b)`,
`encode(book, v)`.

Rust imposes one constraint Go does not: the **orphan rule** forbids an inherent `impl` outside
the type's defining crate. Where behaviour must attach to a type from another crate, define a
**trait** and implement it — that keeps the receiver form:

```rust
pub trait CodebookBytes {
    fn encoded_len(&self) -> usize;
    fn encode_to(&self, out: &mut Vec<u8>);
}
impl CodebookBytes for PqCodebook { /* ... */ }   // book.encode_to(&mut bytes)
```

A free function is right only where there is genuinely no receiver — a private helper inside one
operation, or a constructor-like entry point that belongs to no value yet.

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
- Policy tests exercise **both** enforcement call sites (query planner and inference server) with
  the same `(principal, collection, policy_version)` fixtures, so the two checkpoints can't
  silently drift apart.

**Async.** `tokio` in the server only. Storage and index crates are synchronous and
runtime-agnostic; that keeps them benchable and embeddable. The inference server's scheduler is
the one exception worth naming explicitly: batching across concurrent callers needs an async
runtime, so `telividb-embed`'s scheduler lives behind the same `subscriber`-style feature-gating
pattern as telemetry (rule 41) — synchronous and directly callable when embedded, async-scheduled
under `telividb-server`.

**Config.** One `telividb.toml`, deserialized via `serde` into typed structs in `telividb-core`.
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
- **GGUF is not universal.** It covers the model architectures a loader implements, not every
  model on HuggingFace. Scope is encoder-style embedding models (bge, e5, gte, nomic, jina) plus
  whatever generative or scoring models the model runtime supports for plugin compute (rule 43).
  Which architectures are in reach now depends on *which* model runtime a field uses — rule 42
  permits several above the engine — so check the runtime bound to that field before concluding
  a model is out of scope.
- **A ggml graph is not an eager tensor.** Declaring a tensor does not compute anything; a graph
  is built and dispatched in one call. Writing `a.matmul(b)` and expecting a value back is the
  most natural mistake to make here, and the API deliberately offers whole jobs instead — see
  rule 42.
- **Multi-model VRAM pressure is real and currently undesigned.** Loading an embedder, a rerank
  model, and the OCEAN scoring model simultaneously (rule 45) can exceed available memory on
  smaller GPUs well before it exceeds anything on a datacenter card. There is no eviction policy
  yet — don't assume one exists; surface the gap if you hit it.
- **Jetson is CUDA-on-aarch64.** It is a cross-compilation and driver-version problem, not a code
  problem. Do not claim support without having run it on the device.
- **macOS gets no GPU inside a container.** Not Docker, not Apple's `container` — Apple GPUs have
  no IOMMU and `Hypervisor.framework` exposes no virtual GPU. On macOS the binary runs natively;
  containers are the Linux story. Never write a Metal code path that assumes a container runtime.
- **The desktop app is packaging, not architecture.** If logic ends up in the Tauri layer instead
  of the server, the boundary is wrong — the browser and the app must reach identical behaviour.
- **Only official SDKs for model providers.** `openai`, `@anthropic-ai/sdk`, `@google/genai` and
  `ollama` are each published by the company whose API they call. The Rust options are
  one-provider (`async-openai`) or third-party and stale since 2024, which is why answering is
  TypeScript at all — and, since the window already runs TypeScript, why it needs no sidecar to
  get there. No unifying framework sits over them: the Vercel AI SDK is well made and
  third-party to every provider, and Google's ADK — though genuinely Google's — brings an
  ORM-backed session store to an application whose entire job is storing sessions.
- **Never introduce cross-segment state.** A shared centroid table, a graph spanning segments, an
  ID map consulted across segments — each silently breaks scatter-gather and forecloses clustering
  (AGENT_START.md §14.3). IVF centroids are per-segment or replicated read-only, never mutable and
  shared. This costs nothing today and is expensive to unwind later.
- **The graph is in-memory, full stop.** `petgraph` (rule 47) does not spill to disk. A collection
  with an enormous edge count is a RAM problem before it's a search-quality problem — this is a
  known v1 ceiling, not a bug to quietly patch around with, say, an unbounded cache.
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
- **The OCEAN plugin is not a vetted psychometric instrument.** There is currently no widely
  validated GGUF-native model for trait scoring; the reference plugin likely starts as a
  general-purpose instruction model prompted for structured output. Document it as a pipeline
  proof, not a clinical claim, anywhere it's user-facing.

---

## Working style for this repo

- The user is architecting alongside you. For anything touching the on-disk format, the index
  trait, the inference-server scheduling model, or the proto contract: **propose, get agreement,
  then implement.** These are expensive to reverse.
- Prefer landing a narrow vertical slice that runs end-to-end over a wide layer that runs nowhere.
  The voice and OCEAN reference plugins (in the sibling `telividb-plugins` repo) exist to be
  exactly this — build the engine against what they actually need before generalizing a port.
- Benchmark before optimizing. This codebase will attract premature SIMD; resist it until a
  profile justifies it.
- When a phase from `AGENT_START.md` completes, update the status markers there in the same change.