# Codebase Map for Nullable Column Support

## Key Files and Their Roles

### Type System
- `crates/proof-of-sql/src/base/database/column_type.rs` - `ColumnType` enum defines all supported types (Boolean, TinyInt, SmallInt, Int, BigInt, Int128, Decimal75, VarChar, TimestampTZ, Scalar, VarBinary)
- `crates/proof-of-sql/src/base/database/column_type_operation.rs` - Type coercion and operation result type inference
- `crates/proof-of-sql/src/base/database/literal_value.rs` - Literal values for SQL constants

### Column Storage
- `crates/proof-of-sql/src/base/database/column.rs` - `Column<'a, S>` - read-only view of column data (borrowed)
- `crates/proof-of-sql/src/base/database/owned_column.rs` - `OwnedColumn<S>` - owned column data with Vec storage
- `crates/proof-of-sql/src/base/database/owned_table.rs` - `OwnedTable<S>` - collection of named columns

### Column Operations
- `crates/proof-of-sql/src/base/database/column_arithmetic_operation.rs` - Add, subtract, multiply, divide
- `crates/proof-of-sql/src/base/database/column_comparison_operation.rs` - Equality, inequality
- `crates/proof-of-sql/src/base/database/owned_column_operation.rs` - Operations on OwnedColumn

### Arrow Conversions
- `crates/proof-of-sql/src/base/arrow/arrow_array_to_column_conversion.rs` - Arrow → Column (currently rejects nulls!)
- `crates/proof-of-sql/src/base/arrow/owned_and_arrow_conversions.rs` - OwnedColumn ↔ Arrow

### Commitment/Proof System
- `crates/proof-of-sql/src/base/commitment/committable_column.rs` - Column data in "committable form"
- `crates/proof-of-sql/src/base/commitment/column_commitments.rs` - Commitments for columns
- `crates/proof-of-sql/src/base/commitment/table_commitment.rs` - Table-level commitments
- `crates/proof-of-sql/src/sql/proof/query_proof.rs` - Query proof generation

### Proof Expressions
- `crates/proof-of-sql/src/sql/proof_exprs/add_expr.rs` - Add expression with proof
- `crates/proof-of-sql/src/sql/proof_exprs/subtract_expr.rs` - Subtract expression
- `crates/proof-of-sql/src/sql/proof_exprs/multiply_expr.rs` - Multiply expression
- `crates/proof-of-sql/src/sql/proof_exprs/equals_expr.rs` - Equality expression
- `crates/proof-of-sql/src/sql/proof_exprs/inequality_expr.rs` - Inequality expressions

## Current Null Handling

Currently in `arrow_array_to_column_conversion.rs:112-113`:
```rust
if self.null_count() != 0 {
    return Err(ArrowArrayToColumnConversionError::ArrayContainsNulls);
}
```
Nulls are explicitly rejected!

## Implementation Plan

### Phase 1: Type System
Add nullability tracking:
- Option 1: Add `nullable: bool` flag to ColumnType (changes enum size, affects many places)
- Option 2: Create wrapper `NullableColumnType { base: ColumnType, nullable: bool }`
- Option 3: Create nullable variants in OwnedColumn/Column with validity mask

**Decision**: Use Option 3 - add validity mask to column storage, least invasive to existing type system.

### Phase 2: Column Storage
Add validity mask support:
- `OwnedColumn` variants get `Option<Vec<bool>>` validity
- `Column` variants get `Option<&'a [bool]>` validity
- Helper methods: `is_valid(index)`, `validity_mask()`, `with_validity(mask)`

### Phase 3: Arrow Conversion
- Accept nullable arrays
- Extract validity bitmap
- Enforce canonical null values (0 for numeric, empty for strings)

### Phase 4: Operations
- Null propagation: `NULL op X = NULL`
- Combine validity masks: `and` for binary ops
- WHERE treats NULL as false

### Phase 5: Proof Integration
- Commit validity mask
- Add constraints: `!valid[i] => value[i] == 0`
- Prove null propagation correctly
