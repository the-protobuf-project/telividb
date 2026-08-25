#!/usr/bin/env bash
#
# Fetch the evaluation datasets the benchmarks run against.
#
# Pinned by SHA-256 like the models are, and for the same reason: a benchmark
# is a claim about accuracy, and a claim measured against a dataset that
# silently changed is worse than no claim at all. A number that moved would be
# blamed on the code.
#
#   ./download.sh              # every dataset
#   ./download.sh --list       # what is available
#   ./download.sh scifact      # just one
#
set -euo pipefail

cd "$(dirname "$0")"

# name | sha256 | url
#
# BEIR's own distribution rather than the HuggingFace mirror. The mirror ships
# Parquet, which would pull arrow into the workspace to read three columns;
# this is JSONL, which `serde_json` already handles.
DATASETS="
nfcorpus|efe5be03f8c5b86a5870102d0599d227c8c6e2484328e68c6522560385671b0b|https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/nfcorpus.zip
scifact|536e14446a0ba56ed1398ab1055f39fe852686ecad24a6306c80c490fa8e0165|https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip
arguana|cfdf79adce27a401b3cd3ea267903134dbfab2c6afeb95d7fe5724a00bf7557b|https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/arguana.zip
fiqa|32c7df99ed21252fdfb2cf3f5673502a8d245ee0c44c4a133570d92ce2b3ad02|https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/fiqa.zip
"

# ANN datasets: raw vectors with published ground truth, for the recall-versus-
# QPS curve the field actually compares on. A different shape from BEIR — a
# tarball of `.fvecs` rather than a zip of JSONL.
#
# `.fvecs` deliberately, not the HDF5 the ann-benchmarks site distributes:
# HDF5 would mean linking a C library, and invariant 1 allows exactly two
# native paths — neither of them this. The format is `int32 dim` followed by
# `dim` floats, per vector, which needs no library at all.
ANN_DATASETS="
siftsmall|b8f1e59b20319ac44279d5251706909dd3a5b8ca5ce2a11ddb1e73902252770e|ftp://ftp.irisa.fr/local/texmex/corpus/siftsmall.tar.gz
sift|92f1270c5e3a0cb46b89983e72b0511e4df065c31a9fa0276d8c9b1fca5bc81a|ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz
"

# Four corpora spanning 3.6k to 57.6k documents, which is what makes the
# throughput curve meaningful — a single size cannot show whether cost scales
# with the corpus or with something else.
DEFAULT="all"

digest_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

if [ "${1:-}" = "--list" ] || [ "${1:-}" = "-l" ]; then
  echo "available datasets:"
  echo "  retrieval (BEIR — nDCG/recall against graded judgements):"
  echo "$DATASETS" | while IFS='|' read -r name _ url; do
    [ -z "${name:-}" ] && continue
    echo "    ${name}"
  done
  echo "  ann (recall-versus-QPS against exact ground truth):"
  echo "$ANN_DATASETS" | while IFS='|' read -r name _ url; do
    [ -z "${name:-}" ] && continue
    echo "    ${name}"
  done
  echo
  echo "  'all' fetches the BEIR set; name an ANN dataset explicitly, since"
  echo "  sift is 168 MB."
  exit 0
fi

WANT="${1:-$DEFAULT}"

fetch_one() {
  local want="$1" expected="" url="" kind="beir"
  while IFS='|' read -r name sha src; do
    [ -z "${name:-}" ] && continue
    if [ "$name" = "$want" ]; then
      expected="$sha"
      url="$src"
    fi
  done <<EOF
$DATASETS
EOF

  if [ -z "$expected" ]; then
    while IFS='|' read -r name sha src; do
      [ -z "${name:-}" ] && continue
      if [ "$name" = "$want" ]; then
        expected="$sha"
        url="$src"
        kind="ann"
      fi
    done <<EOF
$ANN_DATASETS
EOF
  fi

  if [ -z "$expected" ]; then
    echo "error: no pinned digest for dataset '${want}'." >&2
    echo "Run '$0 --list' to see what this script can verify." >&2
    return 1
  fi

  if [ "$kind" = "ann" ] && [ -f "${want}/${want}_base.fvecs" ]; then
    echo "  ${want}: already extracted"
    return 0
  fi
  if [ "$kind" = "beir" ] && [ -f "${want}/corpus.jsonl" ]; then
    echo "  ${want}: already extracted ($(wc -l < "${want}/corpus.jsonl" | tr -d ' ') documents)"
    return 0
  fi

  local archive="${want}.zip"
  [ "$kind" = "ann" ] && archive="${want}.tar.gz"

  echo "  ${want}: downloading..."
  # To a temporary name first, so an interrupted run cannot leave a truncated
  # archive where the next run would find it.
  curl -fL --progress-bar -o "${archive}.part" "$url"

  local found
  found="$(digest_of "${archive}.part")"
  if [ "$found" != "$expected" ]; then
    rm -f "${archive}.part"
    echo "error: digest mismatch for ${archive}." >&2
    echo "  expected ${expected}" >&2
    echo "  found    ${found}" >&2
    echo "The download was discarded rather than kept." >&2
    return 1
  fi
  mv "${archive}.part" "$archive"

  if [ "$kind" = "ann" ]; then
    tar xzf "$archive"
    rm -f "$archive"
    echo "  ${want}: verified"
  else
    unzip -oq "$archive"
    rm -f "$archive"
    echo "  ${want}: verified, $(wc -l < "${want}/corpus.jsonl" | tr -d ' ') documents"
  fi
}

if [ "$WANT" = "all" ]; then
  echo "$DATASETS" | while IFS='|' read -r name _ _; do
    [ -z "${name:-}" ] && continue
    fetch_one "$name"
  done
else
  fetch_one "$WANT"
fi

echo
echo "Run the benchmark with:"
echo "  cargo run --release -p telividb-examples --bin beir"
