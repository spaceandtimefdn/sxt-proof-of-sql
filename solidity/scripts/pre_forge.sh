#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

CACHE_FINGERPRINT_FILE=".pre_forge_inputs.sha256"
PREPROCESSOR_SCRIPT="preprocessor/yul_preprocessor.py"

compute_input_fingerprint() {
  python3 - "$PREPROCESSOR_SCRIPT" <<'PY'
from hashlib import sha256
from pathlib import Path
import sys

preprocessor_script = Path(sys.argv[1])
root = Path.cwd()
files = [preprocessor_script, *sorted(root.rglob("*.presl"))]
hasher = sha256()
for path in files:
    stat = path.stat()
    hasher.update(path.as_posix().encode("utf-8"))
    hasher.update(b"\0")
    hasher.update(str(stat.st_size).encode("ascii"))
    hasher.update(b"\0")
    hasher.update(str(stat.st_mtime_ns).encode("ascii"))
    hasher.update(b"\n")
print(hasher.hexdigest())
PY
}

clear_generated_outputs() {
  find . -type f -name '*.post.sol' -delete
  rm -f "$CACHE_FINGERPRINT_FILE"
}

has_missing_generated_outputs() {
  while IFS= read -r -d '' presl_file; do
    local output_file="${presl_file%.presl}.post.sol"
    if [[ ! -f "$output_file" ]]; then
      return 0
    fi
  done < <(find . -type f -name '*.presl' -print0)
  return 1
}

has_stale_generated_outputs() {
  while IFS= read -r -d '' output_file; do
    local source_file="${output_file%.post.sol}.presl"
    if [[ ! -f "$source_file" ]]; then
      return 0
    fi
  done < <(find . -type f -name '*.post.sol' -print0)
  return 1
}

ensure_preprocessed_sources() {
  local current_fingerprint
  current_fingerprint="$(compute_input_fingerprint)"

  if [[ -f "$CACHE_FINGERPRINT_FILE" ]] \
    && [[ "$(cat "$CACHE_FINGERPRINT_FILE")" == "$current_fingerprint" ]] \
    && ! has_missing_generated_outputs \
    && ! has_stale_generated_outputs; then
    echo "Using cached .post.sol files"
    return
  fi

  clear_generated_outputs
  python3 "$PREPROCESSOR_SCRIPT" .
  printf '%s\n' "$current_fingerprint" > "$CACHE_FINGERPRINT_FILE"
}

if [[ "${1:-}" == "clean" ]]; then
  clear_generated_outputs
  exec forge "$@"
fi

ensure_preprocessed_sources
exec forge "$@"
