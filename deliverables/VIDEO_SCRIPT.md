# Demo Video Script: Nullable Column Support

## Overview
Duration: ~2-3 minutes
Purpose: Demonstrate nullable column support working with Proof of SQL

---

## Script

### Scene 1: Introduction (15 seconds)
```
[Screen: Terminal in project directory]
"This demo shows nullable column support for Proof of SQL, implementing Issue #183."
```

### Scene 2: Run Tests (30 seconds)
```
[Type command]
$ cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable --nocapture

[Narration]
"Running all nullable column tests. 21 tests covering validity masks, 
null propagation, Arrow conversion, and proof integration."

[Wait for output showing all tests pass]
```

### Scene 3: Key Test - Nullable + Non-Nullable (30 seconds)
```
[Highlight test output]
test_nullable_plus_nonnullable_bigint_requirement ... ok

[Narration]
"This test specifically verifies the Issue #183 requirement: 
adding a nullable bigint to a non-nullable bigint."
```

### Scene 4: Code Walkthrough - Validity Module (30 seconds)
```
[Open file: crates/proof-of-sql/src/base/database/validity.rs]

[Scroll to combine_validity function]
[Narration]
"The validity module provides mask combination and canonicalization.
combine_validity ANDs two masks for null propagation."
```

### Scene 5: Code Walkthrough - Nullable Column (30 seconds)
```
[Open file: crates/proof-of-sql/src/base/database/nullable_column.rs]

[Scroll to NullableOwnedColumn struct]
[Narration]
"NullableOwnedColumn wraps an OwnedColumn with an optional validity mask.
The canonical null invariant ensures proof soundness."
```

### Scene 6: Arrow Conversion (20 seconds)
```
[Open file: crates/proof-of-sql/src/base/arrow/nullable_conversion.rs]

[Highlight nullable_bigint_from_arrow function]
[Narration]
"Arrow arrays with nulls are converted while preserving validity bitmaps
and enforcing canonical null values."
```

### Scene 7: Conclusion (15 seconds)
```
[Screen: PR page]
"PR #1120 is ready for review. The implementation provides a foundation
for full nullable column support with proof soundness guarantees."
```

---

## Recording Notes

1. Use a clean terminal with large font
2. Pause briefly after each command for readability
3. Highlight relevant code sections
4. Keep narration concise and technical
5. End with PR link visible

## Files to Show

- `crates/proof-of-sql/src/base/database/validity.rs`
- `crates/proof-of-sql/src/base/database/nullable_column.rs`
- `crates/proof-of-sql/src/base/arrow/nullable_conversion.rs`
- Test output showing all 21 tests pass
