# Design Notes: Nullable Column Support

## Current State

### Type System
- `ColumnType` enum with 12 variants (Boolean, Uint8, TinyInt, SmallInt, Int, BigInt, Int128, Decimal75, VarChar, TimestampTZ, Scalar, VarBinary)
- Types are `Copy` and used extensively for type inference
- No nullability tracking

### Column Storage
- `Column<'a, S>` - borrowed view with slices
- `OwnedColumn<S>` - owned data with Vecs
- No validity/null mask support

### Arrow Conversion
- Currently rejects any arrays with nulls (`ArrayContainsNulls` error)
- No validity bitmap handling

### Proof System
- Columns committed via `CommittableColumn`
- No validity mask in commitments

## Proposed Design

### Approach: Validity Mask in Column Storage

Rather than modifying `ColumnType` (which is `Copy` and used everywhere), we add optional validity masks directly to column storage types.

### Type Changes

#### New Type: `ColumnTypeWithNullability`
```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColumnTypeWithNullability {
    pub column_type: ColumnType,
    pub nullable: bool,
}
```

This wrapper is used in schemas/metadata where nullability matters, but `ColumnType` remains unchanged for internal operations.

#### OwnedColumn Changes
Add nullable variants or wrap with validity:

```rust
pub enum OwnedColumn<S: Scalar> {
    // Existing variants unchanged
    Boolean(Vec<bool>),
    BigInt(Vec<i64>),
    // ...
    
    // New nullable variants
    NullableBoolean(Vec<bool>, Vec<bool>),  // (values, validity)
    NullableBigInt(Vec<i64>, Vec<bool>),
    // ... etc
}
```

Alternative (chosen): Use a wrapper struct:
```rust
pub struct NullableOwnedColumn<S: Scalar> {
    pub column: OwnedColumn<S>,
    pub validity: Option<Vec<bool>>,  // None = all valid
}
```

For PoC, we'll start by adding nullable BigInt support directly.

### Canonical Null Invariant

When `validity[i] == false`:
- Numeric types: `value[i] == 0`
- String types: `value[i] == ""`
- Binary types: `value[i] == []`

This is enforced:
1. On construction
2. On Arrow import
3. In proof constraints

### Arrow Conversion Strategy

```rust
// In arrow_array_to_column_conversion.rs
fn to_column_with_validity<'a, S: Scalar>(
    &'a self,
    alloc: &'a Bump,
    range: &Range<usize>,
    scals: Option<&'a [S]>,
) -> Result<(Column<'a, S>, Option<&'a [bool]>), Error> {
    // Extract validity bitmap
    let validity = if self.null_count() > 0 {
        let validity_bits = self.nulls().unwrap();
        // Convert to bool slice, enforce canonical values
        Some(extract_and_canonicalize(validity_bits, values))
    } else {
        None
    };
    // ... rest of conversion
}
```

### Operation Semantics

#### Arithmetic (NULL op X = NULL)
```rust
fn add_nullable<T>(
    lhs: &[T], lhs_valid: Option<&[bool]>,
    rhs: &[T], rhs_valid: Option<&[bool]>,
) -> (Vec<T>, Option<Vec<bool>>) {
    let result_validity = combine_validity(lhs_valid, rhs_valid);
    let values = lhs.iter().zip(rhs).map(|(l, r)| l + r).collect();
    // Canonicalize: set values[i] = 0 where !result_validity[i]
    (values, result_validity)
}
```

#### Comparisons (NULL cmp X = NULL)
Result is nullable boolean.

#### WHERE/Filter
NULL predicate → row excluded (treated as false).

### Proof Strategy

#### Commit Validity
Validity mask committed as a separate column (boolean column):
```rust
// In CommittableColumn
pub enum CommittableColumn<'a> {
    // ... existing
    Validity(&'a [bool]),  // For nullable columns
}
```

#### Constraints
For each nullable column, add constraint:
```
∀i: !validity[i] => value[i] == canonical_default
```

For operations, prove correct null propagation:
```
∀i: result_validity[i] == (lhs_validity[i] AND rhs_validity[i])
```

### Implementation Order

1. **PoC (Commit 1)**: Nullable BigInt
   - Add validity to OwnedColumn::BigInt variant or new NullableBigInt
   - Arrow import with validity
   - Simple add operation with null propagation
   - Commit validity mask
   - One proof test

2. **Full Implementation (Commits 2+)**:
   - Extend to all numeric types
   - All arithmetic operations
   - Comparison operations
   - WHERE handling
   - String/binary null support
   - Comprehensive tests

## Migration Notes

### Breaking Changes
- Arrow conversion API may change signature
- Schema accessors may need nullability info
- Query result types may include validity

### Backward Compatibility
- Non-nullable columns work exactly as before
- Validity mask is optional (None = all valid)
- Existing tests should pass without modification
