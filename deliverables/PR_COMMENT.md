## How to Test

### Quick Verification (PoC with nulls)
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable_column_proof_test --nocapture
```

### Full Nullable Test Suite (31 tests)
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable
```

### Specific Issue #183 Requirement Test
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture

# Linux/CI: end-to-end proof with blitzar
cargo test -p proof-of-sql --features "perf,arrow" test_nullable_bigint_filter_proves_with_validity --nocapture
```

---

## PoC Test File

**Location:** `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs`

Key tests:
- `test_nullable_column_to_committable` - Creates committable columns from nullable data
- `test_canonical_null_invariant_preserved` - Verifies null propagation and canonical values
- `test_nullable_plus_nonnullable_bigint_requirement` - Issue #183 explicit requirement
- `test_nullable_bigint_filter_proves_with_validity` (cfg `blitzar`) - Proof generate/verify with validity filter

Std/arrow suite passes locally; blitzar proof test runs in Linux CI. Ready for review.

---

Nicholas Toledo / Toledo Technologies LLC
