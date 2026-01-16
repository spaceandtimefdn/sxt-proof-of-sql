# Claim Instructions for Nullable Column Support Bounty

## Important: Do NOT Claim Until PR is Merged

The `/claim` command should only be used AFTER the PR has been merged by maintainers.

---

## Pre-Claim Checklist

Before claiming, verify:

- [ ] PR #1120 has been merged to main
- [ ] All CI checks have passed
- [ ] Maintainers have approved the changes
- [ ] No outstanding requested changes

---

## Claim Comment Template

Once PR is merged, post this comment on Issue #183:

```
/claim

## Summary

Implemented nullable column support for Proof of SQL.

### Key Deliverables

1. **Validity Module** (`validity.rs`)
   - Mask combination and canonicalization utilities
   - Enforces canonical null invariant for proof soundness

2. **Nullable Column Types** (`nullable_column.rs`)
   - `NullableOwnedColumn<S>` wrapper with validity mask
   - Operations: add, subtract, multiply with null propagation
   - Support for nullable + non-nullable operations

3. **Arrow Integration** (`nullable_conversion.rs`)
   - Converts nullable Arrow arrays to NullableOwnedColumn
   - Preserves validity bitmaps
   - Enforces canonical null values on import

4. **Tests**
   - 21 tests covering all nullable functionality
   - Proof integration tests demonstrating commitment compatibility
   - Specific test for Issue requirement: nullable + non-nullable bigint

### PR Reference

https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

---

Nicholas Toledo / Toledo Technologies LLC
```

---

## Post-Claim Follow-up

After claiming:

1. Monitor the issue for any maintainer questions
2. Be prepared to make follow-up fixes if requested
3. Keep bounty claim tracking updated

---

## Contact

Nicholas Toledo
Toledo Technologies LLC
