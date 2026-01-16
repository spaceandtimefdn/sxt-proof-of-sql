/claim #183

# Nullable column support PoC

## Summary
- Added an end-to-end nullable proof using `NaiveEvaluationProof` that mixes canonicalized nullable bigint data with a non-nullable column and filters nulls via validity.
- Ensured nullable arithmetic semantics remain (canonicalized nulls) while supporting nullable + non-nullable operations already covered by `add_nullable_to_nonnullable_bigint`.
- Captured reproduction steps and local logs for the nullable test suite and the PoC command.
- Added a Unicode control character scanner (`tools/find_unicode_controls.py`) with a clean report in `deliverables/UNICODE_CONTROL_REPORT.md`.
- Kept changes scoped to the nullable slice to minimize surface area.

## How this satisfies Issue #183
- Nullable mechanism: `NullableOwnedColumn`/`NullableColumn` with validity masks and canonical null enforcement (`crates/proof-of-sql/src/base/database/validity.rs`).
- Operations on nullable columns: arithmetic helpers including nullable + non-nullable (`add_nullable_bigint`, `add_nullable_to_nonnullable_bigint`) with existing unit coverage.
- Non-trivial proof involving a null type: new PoC proves and verifies a query over nullable bigint data plus a non-nullable column, driven by the validity mask.

## Non-trivial proof PoC
- Location: `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs` (`test_nullable_bigint_proof_with_nulls_and_nonnullable_mix`).
- Command: `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_bigint_proof_with_nulls_and_nonnullable_mix --nocapture`.
- Success criteria: proof constructs and verifies, returning only valid rows with summed values (expected totals: 10, 16, 22).

## Review guide
- Start with the new PoC test to see the end-to-end flow and expected result.
- Check `validity.rs` for canonical null handling and `nullable_column.rs` for nullable arithmetic helpers.
- Repro commands are in `deliverables/REPRO_STEPS.md`; quick log in `deliverables/LOCAL_TEST_LOG.txt`.

## Tests
- `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_bigint_proof_with_nulls_and_nonnullable_mix`
- `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable`
