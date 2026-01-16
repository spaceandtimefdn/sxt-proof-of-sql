## PR Ready for Review

**PR:** https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

### How to Test

**PoC Proof Test (with nulls):**
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable_column_proof_test --nocapture
```

**Full Nullable Test Suite (31 tests):**
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable
```

**Linux/CI (blitzar) end-to-end proof:**
```bash
cargo test -p proof-of-sql --features "perf,arrow" test_nullable_bigint_filter_proves_with_validity --nocapture
```

### Design Summary

- **Validity Mask Pattern**: `NullableOwnedColumn<S>` wraps `OwnedColumn` with `Option<Vec<bool>>` validity mask
- **Canonical Null Invariant**: Null positions always contain canonical default (0 for numeric) - critical for proof soundness
- **Null Propagation**: `NULL op X = NULL` via AND-combined validity masks
- **Nullable + Non-nullable**: Explicit support for `add_nullable_to_nonnullable_bigint()` as required

### Files Added

| File | Purpose |
|------|---------|
| `validity.rs` | Mask combination & canonicalization |
| `nullable_column.rs` | `NullableOwnedColumn` type & ops |
| `nullable_column_proof_test.rs` | PoC proof tests |
| `column_type.rs`, `owned_column.rs`, `committable_column.rs` | `NullableBigInt` variants + conversions |
| `nullable_conversion.rs` | Arrow nullable conversion |

---

Nicholas Toledo / Toledo Technologies LLC
