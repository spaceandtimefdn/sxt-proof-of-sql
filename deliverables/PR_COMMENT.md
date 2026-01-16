## How to Test

### Quick Verification (PoC with nulls)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```

### Full Nullable Test Suite (21 tests)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```

### Specific Issue #183 Requirement Test
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```

---

## PoC Test File

**Location:** `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs`

Key tests:
- `test_nullable_column_to_committable` - Creates committable columns from nullable data
- `test_canonical_null_invariant_preserved` - Verifies null propagation and canonical values
- `test_nullable_plus_nonnullable_bigint_requirement` - Issue #183 explicit requirement

All 21 nullable tests pass locally. Ready for review.

---

Nicholas Toledo / Toledo Technologies LLC
