/attempt

## Plan: Add Nullable Column Support

I'm implementing nullable column support for Proof of SQL. Here's my approach:

### Implementation Strategy

1. **PoC First**: Start with a narrow vertical slice for `BigInt` (i64) to demonstrate:
   - Nullable type representation in schema
   - Column storage with validity mask
   - Arrow conversion with null handling
   - One non-trivial proof involving nulls

2. **Type System Changes**:
   - Add nullability as an orthogonal flag to `ColumnType`
   - Create `NullableColumn` wrapper with `(values, Option<validity_mask>)`
   - Enforce canonical null values (0/empty) for soundness

3. **Proof Soundness**:
   - Commit validity mask alongside column data
   - Add constraints enforcing: `null ⇒ value == canonical_default`
   - Ensure null propagation is correctly proven for operations

4. **Operations**:
   - Arithmetic: `NULL op X = NULL`
   - Comparisons: `NULL cmp X = NULL/unknown`
   - WHERE: `NULL` predicate → row filtered (treated as false)
   - Support mixed nullable/non-nullable operands (e.g., nullable bigint + non-nullable bigint)

### How I'll Keep PR Reviewable

- PoC in first commits demonstrating proof with nulls
- Clear commit separation: type system → storage → ops → proof
- Detailed criteria mapping to acceptance requirements
- Comprehensive tests for each component

### Timeline

Opening draft PR as soon as PoC is ready. Not waiting for feedback to continue implementation.

---
Nicholas Toledo / Toledo Technologies LLC
