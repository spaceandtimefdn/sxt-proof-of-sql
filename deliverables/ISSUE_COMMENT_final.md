## PR Submitted: #1120

I've submitted a pull request implementing nullable column support for BigInt:

**PR**: https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

### Implementation Summary

- **`NullableBigInt` variants** across `ColumnType`, `OwnedColumn`, and `CommittableColumn`
- **Validity bitmap pattern**: `(Vec<i64>, Vec<bool>)` where `presence[i] = true` means valid, `false` means null
- **Arrow integration**: Int64 arrays with nulls convert to `NullableBigInt` + validity; Arrow fields marked nullable
- **Proof/commitment plumbing updated** for NullableBigInt (bounds, packing, KZG/Dory helpers)
- **PoC proof**: nullable BigInt filtered by validity and fully proved/verified (cfg `blitzar`)

### Testing

- `cargo test -p proof-of-sql --no-default-features --features "std,arrow" --no-run`
- `cargo test -p proof-of-sql --no-default-features --features "std,arrow" -- nullable_column_proof_test`
- Linux/CI: `cargo test -p proof-of-sql --features "perf,arrow" test_nullable_bigint_filter_proves_with_validity` (blitzar not available on macOS)

### Next Steps (if desired)

The PoC demonstrates the pattern for one type. Extending to other types (NullableInt, NullableBoolean, etc.) follows the same structure.

---
Nicholas Toledo / Toledo Technologies LLC
