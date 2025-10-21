#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
# Remove any existing .post.sol files
find . -type f -name '*.post.sol' -delete
python3 preprocessor/yul_preprocessor.py .
forge "$@"
