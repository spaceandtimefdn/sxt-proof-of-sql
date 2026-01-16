## PR Submitted: #1120

I've submitted a pull request implementing nullable column support for BigInt:

**PR**: https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

### Implementation Summary

- **New `NullableBigInt` variant** added to `ColumnType` and `OwnedColumn`
- **Validity bitmap pattern**: `(Vec<i64>, Vec<bool>)` where `presence[i] = true` means valid, `false` means null
- **Arrow integration**: Int64 arrays with nulls automatically convert to `NullableBigInt`
- **All proof system paths updated** for the new variant

### Testing

7 unit tests covering:
- Direct NullableBigInt creation
- Arrow conversion (with and without nulls)
- Slice/permute operations preserve validity
- Column type properties

```bash
cargo test --no-default-features --features "std,arrow" nullable_column_test
```

### Next Steps (if desired)

The PoC demonstrates the pattern for one type. Extending to other types (NullableInt, NullableBoolean, etc.) follows the same structure.

---
Nicholas Toledo / Toledo Technologies LLC
