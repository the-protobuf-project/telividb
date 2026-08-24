#!/usr/bin/env bash
#
# Fetch the evaluation datasets the benchmarks run against.
#
# Pinned by SHA-256 like the models are, and for the same reason: a benchmark
# is a claim about accuracy, and a claim measured against a dataset that
# silently changed is worse than no claim at all. A number that moved would be
# blamed on the code.
#
#   ./download.sh              # the default dataset
#   ./download.sh --list       # what is available
#   ./download.sh scifact
#
set -euo pipefail

cd "$(dirname "$0")"

# name | sha256 | url
#
# BEIR's own distribution rather than the HuggingFace mirror. The mirror ships
# Parquet, which would pull arrow into the workspace to read three columns;
# this is JSONL, which `serde_json` already handles.
DATASETS="
scifact|536e14446a0ba56ed1398ab1055f39fe852686ecad24a6306c80c490fa8e0165|https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip
"

DEFAULT="scifact"

digest_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

if [ "${1:-}" = "--list" ] || [ "${1:-}" = "-l" ]; then
  echo "available datasets:"
  echo "$DATASETS" | while IFS='|' read -r name _ url; do
    [ -z "${name:-}" ] && continue
    echo "  ${name}"
    echo "    from ${url}"
  done
  exit 0
fi

WANT="${1:-$DEFAULT}"

expected=""
url=""
while IFS='|' read -r name sha src; do
  [ -z "${name:-}" ] && continue
  if [ "$name" = "$WANT" ]; then
    expected="$sha"
    url="$src"
  fi
done <<EOF
$DATASETS
EOF

if [ -z "$expected" ]; then
  echo "error: no pinned digest for dataset '${WANT}'." >&2
  echo "Run '$0 --list' to see what this script can verify." >&2
  exit 1
fi

if [ -d "$WANT" ] && [ -f "${WANT}/corpus.jsonl" ]; then
  echo "${WANT}/ already extracted."
  echo "  corpus  : $(wc -l < "${WANT}/corpus.jsonl" | tr -d ' ') documents"
  echo "  queries : $(wc -l < "${WANT}/queries.jsonl" | tr -d ' ') total"
  exit 0
fi

echo "Downloading ${WANT}..."
# To a temporary name first, so an interrupted run cannot leave a truncated
# archive where the next run would find it.
curl -fL --progress-bar -o "${WANT}.zip.part" "$url"

found="$(digest_of "${WANT}.zip.part")"
if [ "$found" != "$expected" ]; then
  rm -f "${WANT}.zip.part"
  echo "error: digest mismatch for ${WANT}.zip." >&2
  echo "  expected ${expected}" >&2
  echo "  found    ${found}" >&2
  echo "The download was discarded rather than kept." >&2
  exit 1
fi
mv "${WANT}.zip.part" "${WANT}.zip"

# The archive holds a single top-level directory of the same name.
unzip -oq "${WANT}.zip"
rm -f "${WANT}.zip"

echo "Verified and extracted ${WANT}/"
echo "  corpus  : $(wc -l < "${WANT}/corpus.jsonl" | tr -d ' ') documents"
echo "  queries : $(wc -l < "${WANT}/queries.jsonl" | tr -d ' ') total"
echo "  qrels   : ${WANT}/qrels/test.tsv"
echo
echo "Run the benchmark with:"
echo "  cargo run --release -p telividb-examples --bin scifact"
