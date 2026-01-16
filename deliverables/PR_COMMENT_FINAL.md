## How to Test

### Quick Verification (PoC with nulls)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```

### Full Nullable Test Suite (21 tests)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```

### Specific Issue #183 Requirement Test
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```

---

## Key Files

| File | Purpose |
|------|---------|
| `crates/proof-of-sql/src/base/database/validity.rs` | Validity mask utilities |
| `crates/proof-of-sql/src/base/database/nullable_column.rs` | NullableOwnedColumn type |
| `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs` | **PoC proof tests** |
| `crates/proof-of-sql/src/base/arrow/nullable_conversion.rs` | Arrow conversion |

---

## Structure

**PoC is in the first commits**; remainder generalizes the implementation with Arrow conversion and additional test coverage. The core design uses a validity mask pattern that preserves proof soundness through the canonical null invariant.

All 21 nullable tests pass locally. Ready for review.

---

Nicholas Toledo / Toledo Technologies LLC
