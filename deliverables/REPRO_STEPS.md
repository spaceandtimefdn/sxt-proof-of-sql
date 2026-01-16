# Reproduction Steps: Nullable Column Support

## Prerequisites

- Rust 1.70+ (tested with 1.88)
- Git

## Setup

```bash
# Clone the repository
git clone https://github.com/spaceandtimefdn/sxt-proof-of-sql.git
cd sxt-proof-of-sql

# Checkout the feature branch
git checkout fix/nullable-columns-183
```

## Running Tests

### Quick Verification (Nullable Tests Only)

```bash
# Run all nullable column tests (~30 seconds)
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable --nocapture
```

Expected output:
```
running 22 tests
...
test base::database::nullable_column_proof_test::test_nullable_bigint_proof_with_nulls_and_nonnullable_mix ... ok

test result: ok. 22 passed; 0 failed; 0 ignored
```

### Non-trivial proof PoC (nullable + non-nullable mix)

```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_bigint_proof_with_nulls_and_nonnullable_mix --nocapture
```

### Validity Module Tests

```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- validity --nocapture
```

### Full Test Suite

```bash
# Run all proof-of-sql tests (~5-10 minutes)
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test"
```

### Specific Requirement Test

The issue explicitly requires "we should be able to add a nullable bigint to a non-nullable bigint":

```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```

## Code Quality Checks

```bash
# Format check
cargo fmt --all --check

# Clippy (warnings expected from existing code, not new code)
cargo clippy -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- -D warnings
```

## Manual Verification

### 1. Verify Null Propagation

Open `crates/proof-of-sql/src/base/database/nullable_column.rs` and review:
- `add_nullable_bigint()` at line ~263
- `add_nullable_to_nonnullable_bigint()` at line ~335

### 2. Verify Canonical Null Invariant

Open `crates/proof-of-sql/src/base/database/validity.rs` and review:
- `canonicalize_nulls_numeric()` at line ~95
- All null positions forced to 0

### 3. Verify Arrow Conversion

Open `crates/proof-of-sql/src/base/arrow/nullable_conversion.rs` and review:
- `extract_validity()` at line ~42
- `nullable_bigint_from_arrow()` at line ~58

## Key Files to Review

| File | Purpose |
|------|---------|
| `crates/proof-of-sql/src/base/database/validity.rs` | Validity mask utilities |
| `crates/proof-of-sql/src/base/database/nullable_column.rs` | Nullable column types |
| `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs` | Proof integration tests |
| `crates/proof-of-sql/src/base/arrow/nullable_conversion.rs` | Arrow conversion |

## Troubleshooting

### Tests Not Found

Ensure you're on the feature branch:
```bash
git branch  # Should show * fix/nullable-columns-183
```

### Compilation Errors

Ensure features are enabled:
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test"
```

### Slow First Build

First build compiles dependencies (~3-5 minutes). Subsequent builds are faster.
