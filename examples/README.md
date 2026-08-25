# Examples and benchmarks

Runnable programs that exercise telividb end to end — and, in the case of
`beir`, measure whether it is actually *correct* rather than merely running.

```
examples/
├─ models/      GGUF models, fetched and checksum-verified by download.sh
├─ datasets/    BEIR evaluation datasets, likewise
└─ rust/        the programs themselves, one binary per example
```

Nothing under `models/` or `datasets/` is committed — the smallest useful model
is 80 MiB — so both directories have a `download.sh` that fetches and verifies.

---

## Quick start

```bash
examples/models/download.sh          # nomic-embed-text-v1.5, ~80 MiB
examples/datasets/download.sh        # four BEIR datasets, ~25 MiB

cargo run --release -p telividb-examples --bin semantic_search_grpc
```

`--release` is not optional advice. A debug build runs the encoder's matmuls
unoptimised and takes minutes where release takes seconds.

---

## The examples

| Binary | What it demonstrates |
|---|---|
| `semantic_search` | The embedded path: model, storage and index in one process, no server. |
| `semantic_search_grpc` | The served path. Starts a server, then talks to it through the SDK — and stays running so you can query it from Postman or grpcurl. |
| `gpu_memory` | CRUD against `GpuFlatIndex` at increasing corpus sizes, reporting where the GPU budget stops it. |
| `beir` | Accuracy and throughput across several standard datasets. See below. |

### Querying from Postman

`semantic_search_grpc` seeds a collection and then stays up on `127.0.0.1:7700`,
printing everything needed to query it from elsewhere. Server reflection is
enabled, so Postman can list the services itself rather than being handed the
`.proto` files.

```bash
grpcurl -plaintext -d '{
  "parent": "collections/documents",
  "field_id": "text",
  "query_text": "Will it rain this weekend?",
  "page_size": 3
}' 127.0.0.1:7700 telividb.point.v1.Points/SearchPoints
```

Note `query_text` rather than `query`. Both exist and exactly one may be set:
`query` carries a base64 `bytes` field of raw little-endian `f32`, which is
awkward to type by hand, while `query_text` lets the *server* encode it with
the model the field is bound to. That asymmetry is deliberate — see
[Why the server embeds](#why-the-server-embeds).

---

## The `beir` benchmark

```bash
examples/datasets/download.sh
cargo run --release -p telividb-examples --bin beir

cargo run --release -p telividb-examples --bin beir -- scifact nfcorpus
TELIVIDB_MAX_TOKENS=512 cargo run --release -p telividb-examples --bin beir
```

### What it is for

A twelve-sentence corpus proves the pipeline runs. It cannot prove it is
correct, and that distinction is the whole reason this exists.

An encoder with a subtly wrong tokenizer, pooling mode or rotation convention
returns vectors that are the right width, unit length, finite and entirely
plausible. Nothing errors. Two such bugs were found while building this crate,
and **both passed every unit test** — one was caught only because a toy query
about rain returned a document about Rust.

A standard dataset with graded relevance judgements makes that unmissable.
nDCG@10 on SciFact is directly comparable to the figure published for the same
model, so a broken encoder cannot hide.

### Datasets

Four corpora spanning 3.6k to 57.6k documents. One size cannot show whether
cost scales with the corpus or with something else.

| Dataset | Documents | Judged queries | Domain |
|---|---:|---:|---|
| `nfcorpus` | 3,633 | 323 | Medical / nutrition |
| `scifact` | 5,183 | 300 | Scientific claim verification |
| `arguana` | 8,674 | 1,406 | Argument retrieval |
| `fiqa` | 57,638 | 648 | Financial question answering |

All are pinned by SHA-256. A benchmark is a claim about accuracy, and a claim
measured against a dataset that silently changed is worse than no claim — the
number that moved would be blamed on the code.

### Metrics

**nDCG@10** is the headline, and what published figures quote. It rewards
putting relevant documents *early*, not merely retrieving them. Implemented
with the exponential gain `(2^grade - 1) / log2(rank + 2)` that BEIR and MTEB
use; the linear variant gives different numbers for the same ranking, so mixing
them would make any comparison meaningless.

**Recall@10 and Recall@100** are reported beside it because they fail
differently. An encoder that finds the right documents but orders them badly
shows healthy recall and poor nDCG — which points at pooling or normalisation
rather than at the model. Both low is an encoding problem.

Interpret recall against the dataset. `nfcorpus` averages 38 relevant documents
per query, so Recall@10 there is capped near 0.26 by construction — a low
number is not a failure.

Search is **exhaustive** (`GpuFlatIndex`), so this measures the *encoder*, not
an approximate index. ANN recall is a separate question, answered by
`cargo run -p telividb-index --bin recall`.

### The correctness floor

The benchmark exits non-zero below nDCG@10 of 0.20, so it can gate a change
rather than merely inform one.

That is a **correctness floor, not a quality target**, and it is set far below
what a healthy model scores. Its job is catching a broken encoder: the
tokenizer bug found while building this crate would have landed near 0.2. To
judge quality rather than correctness, compare the measured number against the
MTEB leaderboard entry for the same model.

### GPU memory and leak detection

Every run reports two independent numbers per dataset:

- **reserved** — what this process believes it holds, via the residency registry
- **allocated** — what Metal's `currentAllocatedSize` says the driver holds

A leak is precisely where those disagree over time. One number alone cannot
show it: the registry would look tidy while the device filled up.

Each dataset's index is dropped before its sample is taken, so **reserved should
return to baseline**. Allocated need not — the Metal driver caches buffers
rather than releasing them eagerly, and a modest steady figure is normal. What
matters is whether it keeps climbing in step with the corpora or levels off.

---

## Why the server embeds

The SDK sends text; the server turns it into vectors. That is a guarantee, not
a convenience.

A vector field is bound to one model identity — the SHA-256 of its GGUF. Vectors
from two different models merged into one index do not fail: recall degrades and
every neighbour returned stays plausible. A client holding its own model would
be exactly that second source. Keeping inference server-side also leaves one
place where a policy check at the inference boundary can attach, and a check a
caller can route around is not a check.

A server started without `--model` refuses text with a message naming the flag,
rather than accepting it and silently storing nothing.

---

## Reproducing

Results depend on hardware, model quantisation and sequence-length cap. Record
all three or the numbers mean nothing:

```bash
examples/models/download.sh Q8_0        # a different quantisation
TELIVIDB_MAX_TOKENS=512 ...             # a different cap
```

`TELIVIDB_MAX_TOKENS` is the single biggest throughput lever, because attention
is quadratic in sequence length — halving the cap quarters the attention work.
BEIR's own evaluations cap BERT-family models at 512. Whether that costs
accuracy is a question for the corpus, which is why it is a knob rather than a
default.

## Troubleshooting

**"no GGUF model found"** — run `examples/models/download.sh`.

**"no scifact dataset"** — run `examples/datasets/download.sh`.

**Throughput far below the table** — check the reported device. If it says
`cpu`, no GPU backend initialised, and results are correct but far slower.
On Linux, CUDA is opt-in: `--features cuda`, which needs the CUDA SDK.

**"Database already open"** — `redb` takes an exclusive file lock. Two servers
cannot share a `--data-dir`.
