#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
cd "$SCRIPT_DIR/.."

CACHE_DIR=.pre_forge_cache
CACHE_FILE="$CACHE_DIR/input_fingerprint.sha256"

compute_input_fingerprint() {
  {
    printf '%s\0' ./preprocessor/yul_preprocessor.py ./scripts/pre_forge.sh
    find . -type f \( -name '*.presl' -o \( -name '*.sol' ! -name '*.post.sol' \) \) -print0
  } | sort -z | xargs -0 sha256sum | sha256sum | awk '{print $1}'
}

should_skip_presl_file() {
  local source_file=$1
  head -n 10 "$source_file" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]' | grep -Eq 'does-not-compile|doesnotcompile'
}

generated_outputs_present() {
  while IFS= read -r -d '' source_file; do
    if should_skip_presl_file "$source_file"; then
      continue
    fi

    if [[ ! -f "${source_file%.presl}.post.sol" ]]; then
      return 1
    fi
  done < <(find . -type f -name '*.presl' -print0)
}

if [[ "${1:-}" == "clean" ]]; then
  find . -type f -name '*.post.sol' -delete
  rm -rf "$CACHE_DIR"
  forge "$@"
  exit 0
fi

input_fingerprint=$(compute_input_fingerprint)
if [[ -f "$CACHE_FILE" ]] && [[ "$(<"$CACHE_FILE")" == "$input_fingerprint" ]] && generated_outputs_present; then
  echo "pre_forge cache hit: skipping preprocessing"
else
  echo "pre_forge cache miss: regenerating .post.sol files"
  find . -type f -name '*.post.sol' -delete
  python3 preprocessor/yul_preprocessor.py .
  mkdir -p "$CACHE_DIR"
  printf '%s\n' "$input_fingerprint" > "$CACHE_FILE"
fi

forge "$@"
