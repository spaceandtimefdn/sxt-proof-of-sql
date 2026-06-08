#!/usr/bin/env bash
# Pre-warm the Dory test setup cache before running the test suite.
# This is optional but recommended for CI and first-time local runs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export DORY_TEST_CACHE_DIR="${DORY_TEST_CACHE_DIR:-$PROJECT_ROOT/crates/proof-of-sql/.test-setup-cache}"

echo "=== Dory Test Setup Cache Warmup ==="
echo "Cache dir: $DORY_TEST_CACHE_DIR"
echo ""

mkdir -p "$DORY_TEST_CACHE_DIR"

# Run a single test that exercises setup generation to populate cache
# This is faster than running a dedicated binary since it reuses build artifacts
cd "$PROJECT_ROOT"

# Run the setup test which exercises PublicParameters generation
cargo test \
    --package proof-of-sql \
    --all-features \
    -- proof_primitive::dory::test_setup_accessor::tests::test_cache_roundtrip \
    2>/dev/null && echo "Cache warmed successfully" || echo "Note: Run after initial build"

echo ""
echo "Cache files:"
ls -la "$DORY_TEST_CACHE_DIR/" 2>/dev/null || echo "  (none yet - will be generated on first test run)"
