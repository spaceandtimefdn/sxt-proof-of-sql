## PR Ready for Review

**PR:** https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

### How to Test

**PoC Proof Test (with nulls):**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```

**Full Nullable Test Suite (21 tests):**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
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
| `nullable_conversion.rs` | Arrow nullable conversion |

---

Nicholas Toledo / Toledo Technologies LLC
