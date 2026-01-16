# Add Nullable Column Support

## Summary

This PR implements nullable column support for Proof of SQL, addressing Issue #183. The implementation provides:

- **Type-safe nullable column representation** with validity masks
- **Canonical null invariant** for proof soundness (null values = 0 for numeric types)
- **Null propagation** for arithmetic operations (NULL op X = NULL)
- **Arrow integration** for converting nullable Arrow arrays
- **Support for nullable + non-nullable operations** (explicit requirement from issue)

## Design Overview

### Approach: Validity Mask Pattern

Rather than modifying the existing `ColumnType` enum (which is `Copy` and widely used), this implementation adds:

1. **`validity` module** - Utilities for combining and canonicalizing validity masks
2. **`nullable_column` module** - `NullableOwnedColumn` and `NullableColumn` wrapper types
3. **`nullable_conversion` module** - Arrow array to nullable column conversion

### Key Components

#### Validity Mask (`crates/proof-of-sql/src/base/database/validity.rs`)
- `combine_validity()` - AND-combines validity masks for binary operations
- `canonicalize_nulls_numeric()` - Ensures null positions have canonical values
- `has_nulls()`, `null_count()` - Validity inspection utilities

#### Nullable Column Types (`crates/proof-of-sql/src/base/database/nullable_column.rs`)
- `NullableOwnedColumn<S>` - Wraps `OwnedColumn` with optional validity mask
- `NullableColumn<'a, S>` - Borrowed view with validity
- `add_nullable_bigint()`, `add_nullable_to_nonnullable_bigint()` - Operations with null propagation

#### Arrow Conversion (`crates/proof-of-sql/src/base/arrow/nullable_conversion.rs`)
- `extract_validity()` - Gets validity bitmap from Arrow array
- `nullable_bigint_from_arrow()` - Converts Int64Array to NullableOwnedColumn

### Canonical Null Invariant (Proof Soundness)

For proof soundness, when a value is NULL (validity[i] == false), the corresponding value slot contains a canonical default:
- **Numeric types**: 0
- **String types**: empty string
- **Binary types**: empty slice

This invariant is enforced:
1. At construction (`new_with_canonical_nulls()`)
2. During Arrow import
3. In operation results

This prevents provers from hiding arbitrary values under NULL entries.

### Null Propagation Semantics

| Operation | Behavior |
|-----------|----------|
| `NULL + X` | NULL |
| `X + NULL` | NULL |
| `NULL + NULL` | NULL |
| `nullable + non_nullable` | nullable result |

## PoC Demonstration

The first commits demonstrate a working proof-compatible nullable column:

1. **Validity module** with mask combination and canonicalization
2. **NullableOwnedColumn** type with BigInt operations
3. **Proof integration tests** showing commitment compatibility
4. **Arrow conversion** for nullable arrays

## Test Commands

```bash
# Run nullable column tests
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable

# Run validity tests
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- validity

# Run all tests
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test"
```

## Files Changed

### New Files
- `crates/proof-of-sql/src/base/database/validity.rs` - Validity mask utilities
- `crates/proof-of-sql/src/base/database/nullable_column.rs` - Nullable column types
- `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs` - Proof integration tests
- `crates/proof-of-sql/src/base/arrow/nullable_conversion.rs` - Arrow nullable conversion

### Modified Files
- `crates/proof-of-sql/src/base/database/mod.rs` - Module exports
- `crates/proof-of-sql/src/base/arrow/mod.rs` - Module exports

## Acceptance Criteria Mapping

See `CRITERIA_MAPPING.md` for detailed mapping of implementation to acceptance criteria.

## Future Work

This PoC establishes the foundation. Full implementation would extend to:
- All numeric types (TinyInt, SmallInt, Int, Int128, Decimal75)
- String/Binary nullable support
- Comparison operations with nullable semantics
- WHERE clause NULL handling
- Full proof constraint generation for validity masks

## Author

Nicholas Toledo / Toledo Technologies LLC
