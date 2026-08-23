# episteme

A multimodal **vector and graph database** written in Rust. Bring your own
embedding model, bring your own search algorithm.

> [!WARNING]
> **Pre-alpha. Nothing here is usable yet.** There is no server, no gRPC
> endpoint, and no way to store a vector on disk. What exists is the storage
> spine — segment header, write-ahead log, manifest — and an exhaustive index
> that searches in memory. The on-disk format is **not stable** and will change
> without migration until v0.1 is tagged.

---

## What it is meant to be

Two properties define it; the rest of the architecture follows from them.

**Bring your own embedding model.** Point a config file at a GGUF file.
Inference runs *inside* the database, on whatever accelerator the host has —
Metal, CUDA, Jetson, Intel, or plain CPU — without recompiling anything.

**Bring your own search algorithm.** ANN indexes sit behind a trait. The
bundled ones are not privileged.

Text, images, audio and video share one store, joined by a property graph. A
query can match a transcript and traverse to the video frame that was on screen
while it was spoken.

**[Read the architecture →](./docs/ARCHITECTURE.md)** · **[Explore the segment format →](./docs/explorer.html)**

## Status

| Crate | State |
|---|---|
| `episteme-core` | Domain types, AIP-122 resource names, spans, content refs, **collection schema + additive-compatibility rules**, `SchemaReader` / `VectorStore` ports |
| `episteme-distance` | Scalar dot / L2 / normalize. **No SIMD yet** |
| `episteme-index` | `VectorIndex` port, exhaustive + **HNSW** (persistable), **two-tier scan + rerank**, buffer/segment merge with provenance, recall harness |
| `episteme-storage` | Segment + field headers with schema/model fingerprints, segment writer + reader, **int8, f16, binary and PQ quantization**, WAL with torn-tail recovery, atomic manifest, searchable mutable buffer |
| `episteme-telemetry` | Span/metric vocabulary, redaction, Prometheus + structured logs |
| `episteme-server` | Not started |
| `episteme-embed` | Not started |

Roughly Phase 1 of [eleven](./AGENT_START.md#17-roadmap). Phases 1 and 2 are the
real project; everything after is comparatively mechanical.

## Build

No C toolchain, no CMake, no CUDA SDK. That is a rule, not a coincidence —
`cargo build` must work on a clean machine.

```bash
cargo build --workspace
cargo test  --workspace          # 336 tests
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask check-len            # no file over 200 lines

# Recall against exhaustive search — the only number that makes an ANN change mean anything
cargo run --release -p episteme-index --bin recall -- --rows 20000 --dim 128 --ef 64
```

Toolchain is **pinned to Rust 1.98.0** in `rust-toolchain.toml`; rustup installs
it automatically on clone. A compiler upgrade is a reviewed commit, not something
that happens to whoever built most recently.

If `rustc --version` disagrees with the pin, a Homebrew Rust is shadowing
rustup's shims — see [`docs/REPO_SETUP.md`](./docs/REPO_SETUP.md).

CI builds and tests on **Linux x86_64**, **Linux ARM64** and **macOS ARM64**.
Windows is out of scope. Repository settings that CI depends on are documented in
[`docs/REPO_SETUP.md`](./docs/REPO_SETUP.md).

## Design rules

The full set lives in [`CLAUDE.md`](./CLAUDE.md). The ones that shape everything
else:

- **Sealed segments are immutable.** This is what buys lock-free reads, safe
  `mmap`, free snapshots, replication by file copy, and sharding later.
- **The read path is zero-copy.** Fixed stride, 64-byte aligned, so an mmap'd
  region casts straight to a float slice and feeds SIMD.
- **Authorization is a pre-filter, never a post-filter.** Searching then
  discarding leaks the existence, count and ranking of rows the caller cannot
  see. It is a correctness property, not a performance one.
- **Never embed what must stay secret.** A vector computed from sensitive text
  leaks that text regardless of what guards the payload. Redaction happens
  *before* the embedder — no cryptography undoes it afterwards.
- **Every on-disk structure is versioned**, and an unknown version is refused
  rather than guessed at.
- **No file exceeds 200 lines.** Enforced in CI.
- **Telemetry never emits a vector, a payload, or a vault name.** Logs are read by
  people granted nothing, and a query vector can be inverted toward its source
  text. Shape, never values — with a regression test to prove it.

## Licence

Dual-licensed.

- **[AGPL-3.0-or-later](./LICENSE)** — free for open-source work, forever.
- **[Commercial](./LICENSE-COMMERCIAL.md)** — for proprietary or closed-source
  products, and for offering a network service without publishing your source.

Client SDKs and `.proto` definitions are **Apache-2.0**, so a proprietary
application that merely *talks to* an episteme server needs no commercial
licence. The line is drawn at linking against or modifying the engine.

Note that episteme is designed to be embedded, and **static linking counts** —
a closed-source binary that links the engine needs a commercial licence.

## Contributing

Read [`CLA.md`](./CLA.md) first — a contributor licence agreement is required,
because dual-licensing is impossible without one. You keep your copyright; it
grants a licence, it is not an assignment. Sign off your commits with
`git commit -s`.

Before opening a pull request: `cargo fmt --all`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check-len`.

Any change to an approximate index must report **recall@k against the exhaustive
index**. "It's faster" without a recall number is not a result.
