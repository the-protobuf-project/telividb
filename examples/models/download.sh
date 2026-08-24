#!/usr/bin/env bash
#
# Fetch the GGUF models the Rust examples run against.
#
# Models are not committed — the smallest useful one is 80 MiB — so this
# script is the reproducible way to get them. Every file is checked against a
# pinned SHA-256 *before* it is accepted, because a model's digest is its
# identity in this system (CLAUDE.md rule 12): a truncated or substituted
# download would otherwise register under a name it does not match, and the
# first sign of trouble would be quietly degraded recall.
#
#   ./download.sh              # every model in the default category
#   ./download.sh --list       # what is available
#   ./download.sh text         # every model in one category
#   ./download.sh text nomic-embed-text-v1.5 Q8_0
#
# Files land in `gguf/<category>/`, one directory per modality. The split
# exists so that adding an image or audio model later is a new directory
# rather than a rename of everything already here — and so an example can ask
# for "a text embedder" without knowing which one is present.
#
set -euo pipefail

cd "$(dirname "$0")"

# category | model | repo | quantization:sha256 ...
#
# The digest of each file, read from the HuggingFace API. Adding a model means
# adding a line; adding a quantization means adding a `quant:digest` pair, so
# the verification below can never be silently skipped.
#
# nomic-embed-text-v1.5, not v2-moe. v2 is a mixture-of-experts model and
# candle implements no MoE *embedding* architecture, so it cannot run here at
# all — CLAUDE.md rule 42 rules out adding a second runtime to reach it. v1.5
# is the same family, runs on the in-tree nomic-bert encoder, and produces
# 768-dimensional vectors.
MODELS="
text|nomic-embed-text-v1.5|nomic-ai/nomic-embed-text-v1.5-GGUF|Q4_K_M:d4e388894e09cf3816e8b0896d81d265b55e7a9fff9ab03fe8bf4ef5e11295ac Q8_0:3e24342164b3d94991ba9692fdc0dd08e3fd7362e0aacc396a9a5c54a544c3b7 f16:f7af6f66802f4df86eda10fe9bbcfc75c39562bed48ef6ace719a251cf1c2fdb
"

# Q4_K_M by default: 80 MiB, and the accuracy loss against f16 is immaterial
# for a walkthrough. A user comparing recall across quantizations is exactly
# who should pass an argument.
DEFAULT_QUANT="Q4_K_M"
DEFAULT_CATEGORY="text"

digest_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

list_models() {
  echo "available models:"
  echo "$MODELS" | while IFS='|' read -r category model repo quants; do
    [ -z "${category:-}" ] && continue
    echo
    echo "  ${category}/${model}"
    echo "    from ${repo}"
    printf "    quantizations:"
    for pair in $quants; do printf " %s" "${pair%%:*}"; done
    echo
  done
  echo
  echo "Files are written to gguf/<category>/."
}

fetch_one() {
  local category="$1" model="$2" repo="$3" quant="$4" expected="$5"
  local dir="gguf/${category}"
  local file="${dir}/${model}.${quant}.gguf"

  mkdir -p "$dir"

  if [ -f "$file" ]; then
    # Re-verified rather than assumed present: a partial download from an
    # interrupted run leaves a file of exactly the wrong kind — one that exists.
    if [ "$(digest_of "$file")" = "$expected" ]; then
      echo "  ${file} already present and verified."
      return 0
    fi
    echo "  ${file} does not match its pinned digest; re-downloading."
    rm -f "$file"
  fi

  echo "  downloading ${model}.${quant}.gguf from ${repo}..."
  # To a temporary name first, so an interrupted run cannot leave a truncated
  # file at the real path where the next run would find it.
  curl -fL --progress-bar -o "${file}.part" \
    "https://huggingface.co/${repo}/resolve/main/${model}.${quant}.gguf"

  local found
  found="$(digest_of "${file}.part")"
  if [ "$found" != "$expected" ]; then
    rm -f "${file}.part"
    echo "error: digest mismatch for ${file}." >&2
    echo "  expected ${expected}" >&2
    echo "  found    ${found}" >&2
    echo "The download was discarded rather than kept." >&2
    return 1
  fi

  mv "${file}.part" "$file"
  echo "  verified ${file}"
}

case "${1:-}" in
  --list|-l) list_models; exit 0 ;;
esac

WANT_CATEGORY="${1:-$DEFAULT_CATEGORY}"
WANT_MODEL="${2:-}"
WANT_QUANT="${3:-$DEFAULT_QUANT}"

matched=0
while IFS='|' read -r category model repo quants; do
  [ -z "${category:-}" ] && continue
  [ "$category" = "$WANT_CATEGORY" ] || continue
  [ -n "$WANT_MODEL" ] && [ "$model" != "$WANT_MODEL" ] && continue

  expected=""
  for pair in $quants; do
    case "$pair" in "${WANT_QUANT}:"*) expected="${pair#*:}" ;; esac
  done

  if [ -z "$expected" ]; then
    echo "error: no pinned digest for ${model} at quantization '${WANT_QUANT}'." >&2
    echo "Run '$0 --list' to see what this script can verify." >&2
    echo >&2
    echo "Adding one is deliberate: read its sha256 from the HuggingFace API" >&2
    echo "and record it in MODELS above, so the check stays real rather than" >&2
    echo "skipped." >&2
    exit 1
  fi

  echo "${category}/${model} (${WANT_QUANT}):"
  fetch_one "$category" "$model" "$repo" "$WANT_QUANT" "$expected"
  matched=$((matched + 1))
done <<EOF
$MODELS
EOF

if [ "$matched" -eq 0 ]; then
  echo "error: nothing matched category '${WANT_CATEGORY}'${WANT_MODEL:+ model '${WANT_MODEL}'}." >&2
  echo "Run '$0 --list' to see what is available." >&2
  exit 1
fi

echo
echo "Run the walkthrough with:"
echo "  cargo run --release -p telividb-examples --bin semantic_search"
