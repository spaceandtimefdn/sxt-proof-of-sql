# Criteria Mapping: Nullable Column Support (Issue #183)

## Acceptance Criteria from Issue

### 1. "Support adding a flag (or some other mechanism) to the columns types in order to support nullable columns"

| Requirement | Implementation | Files | Tests |
|------------|----------------|-------|-------|
| Nullability mechanism | `NullableOwnedColumn` wrapper with `Option<Vec<bool>>` validity mask | `nullable_column.rs` | `test_nullable_column_creation` |
| Type tracking | `is_nullable()`, `validity()` methods | `nullable_column.rs:93-102` | `test_non_nullable_column` |
| Schema integration | `ColumnType::NullableBigInt` + `OwnedColumn::NullableBigInt` + `CommittableColumn::NullableBigInt` | `column_type.rs`, `owned_column.rs`, `committable_column.rs` | Commitment + proof tests |
| Arrow interop | NullableBigInt -> Arrow Int64 with validity mask | `column_arrow_conversions.rs`, `owned_and_arrow_conversions.rs` | New conversions compiled |
| Schema docs | `ColumnTypeWithNullability` design (DESIGN_NOTES.md) | DESIGN_NOTES.md | N/A |

### 2. "Support existing operations on nullable columns (e.g. we should be able to add a nullable bigint to a non-nullable bigint)"

| Requirement | Implementation | Files | Tests |
|------------|----------------|-------|-------|
| Nullable + Nullable | `add_nullable_bigint()` | `nullable_column.rs:263-295` | `test_add_nullable_bigint` |
| Nullable + Non-nullable | `add_nullable_to_nonnullable_bigint()` | `nullable_column.rs:335-373` | `test_nullable_plus_nonnullable_bigint_requirement` |
| Null propagation | Result validity = AND of operand validities | `validity.rs:31-45` | `test_null_propagation_chain` |

### 3. "Proof of Concept should demonstrate some non-trivial proof involving at least one null type"

| Requirement | Implementation | Files | Tests |
|------------|----------------|-------|-------|
| PoC proof test | Nullable column commitment and operations | `nullable_column_proof_test.rs` | Commitment tests |
| End-to-end proof | Nullable BigInt filtered by validity, proved + verified | `nullable_column_proof_test.rs` | `test_nullable_bigint_filter_proves_with_validity` (cfg `blitzar`, runs in Linux CI) |
| Committable column | `CommittableColumn::Boolean` for validity mask | `nullable_column_proof_test.rs:33-40` | `test_nullable_column_to_committable` |
| Canonical null values | Values at null positions = 0 | `nullable_column.rs:145-167` | `test_canonical_nulls` |

## Proof Soundness Requirements

| Requirement | Implementation | Files | Tests |
|------------|----------------|-------|-------|
| Validity committed | Validity mask as `CommittableColumn::Boolean` | `nullable_column_proof_test.rs` | `test_nullable_column_to_committable` |
| Canonical null invariant | `canonicalize_nulls_numeric()` | `validity.rs:95-108` | `test_canonicalize_nulls_numeric` |
| Canonical enforcement | `new_with_canonical_nulls()` | `nullable_column.rs:65-73` | `test_canonical_null_invariant_preserved` |
| No hidden values | Null positions forced to 0 | `nullable_column.rs:285-292` | `test_all_null_column` |

## Implementation Coverage

### Type Support

| Type | Implemented | Tests |
|------|-------------|-------|
| BigInt (i64) | ✅ Full | Multiple |
| Int (i32) | ✅ Canonicalize | `canonicalize_nulls_numeric` |
| SmallInt (i16) | ✅ Canonicalize | N/A |
| TinyInt (i8) | ✅ Canonicalize | N/A |
| Int128 | ✅ Canonicalize | N/A |
| Uint8 | ✅ Canonicalize | N/A |
| Decimal75 | 🔲 Future | N/A |
| VarChar | 🔲 Future | N/A |
| Boolean | 🔲 Future | N/A |

### Operation Support

| Operation | Implemented | Tests |
|-----------|-------------|-------|
| Add (nullable + nullable) | ✅ | `test_add_nullable_bigint` |
| Add (nullable + non-nullable) | ✅ | `test_nullable_plus_nonnullable_bigint_requirement` |
| Subtract | ✅ | `subtract_nullable_bigint()` |
| Multiply | ✅ | `test_multiply_nullable_bigint` |
| Divide | 🔲 Future | N/A |
| Comparison | 🔲 Future | N/A |

### Arrow Integration

| Feature | Implemented | Tests |
|---------|-------------|-------|
| Extract validity | ✅ | `test_extract_validity_with_nulls` |
| Convert Int64Array | ✅ | `test_nullable_bigint_from_arrow_with_nulls` |
| Slice conversion | ✅ | `test_nullable_bigint_from_arrow_slice` |
| Canonical null on import | ✅ | `test_nullable_bigint_from_arrow_all_nulls` |

## Test Coverage Summary

| Module | Test Count | Status |
|--------|------------|--------|
| `validity` | 8 | ✅ All pass |
| `nullable_column` | 6 | ✅ All pass |
| `nullable_column_proof_test` | 9 (2 cfg `blitzar`) | ✅ Std/arrow pass; blitzar suite runs in Linux CI |
| `nullable_conversion` | 8 | ✅ All pass |
| **Total** | **31** | ✅ Std/arrow pass |

## Commands to Verify

```bash
# Compile + run nullable suite without blitzar
cargo test -p proof-of-sql --no-default-features --features="std,arrow" --no-run
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable_column_proof_test

# All validity tests  
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- validity

# Specific requirement test (blitzar/Linux)
cargo test -p proof-of-sql --features "perf,arrow" test_nullable_bigint_filter_proves_with_validity
```
