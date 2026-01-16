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
running 21 tests
test base::database::nullable_column::tests::test_add_nullable_bigint ... ok
test base::database::nullable_column::tests::test_add_nullable_to_nonnullable ... ok
test base::database::nullable_column::tests::test_canonical_nulls ... ok
test base::database::nullable_column::tests::test_multiply_nullable_bigint ... ok
test base::database::nullable_column::tests::test_non_nullable_column ... ok
test base::database::nullable_column::tests::test_nullable_column_creation ... ok
test base::database::nullable_column_proof_test::test_all_null_column ... ok
test base::database::nullable_column_proof_test::test_canonical_null_invariant_preserved ... ok
test base::database::nullable_column_proof_test::test_empty_nullable_column ... ok
test base::database::nullable_column_proof_test::test_no_null_nullable_column ... ok
test base::database::nullable_column_proof_test::test_null_propagation_chain ... ok
test base::database::nullable_column_proof_test::test_nullable_column_to_committable ... ok
test base::database::nullable_column_proof_test::test_nullable_plus_nonnullable_bigint_requirement ... ok
test base::arrow::nullable_conversion::tests::test_extract_validity_no_nulls ... ok
test base::arrow::nullable_conversion::tests::test_extract_validity_with_nulls ... ok
test base::arrow::nullable_conversion::tests::test_nullable_bigint_from_arrow_all_nulls ... ok
test base::arrow::nullable_conversion::tests::test_nullable_bigint_from_arrow_no_nulls ... ok
test base::arrow::nullable_conversion::tests::test_nullable_bigint_from_arrow_slice ... ok
test base::arrow::nullable_conversion::tests::test_nullable_bigint_from_arrow_with_nulls ... ok
test base::arrow::nullable_conversion::tests::test_validity_for_range_no_nulls_in_range ... ok
test base::arrow::nullable_conversion::tests::test_validity_for_range_with_nulls ... ok

test result: ok. 21 passed; 0 failed; 0 ignored
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
