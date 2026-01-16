Heads up on the nullable PoC:
- Added `test_nullable_bigint_proof_with_nulls_and_nonnullable_mix` in `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs` (proves nullable bigint + non-nullable column with validity filtering).
- Command: `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_bigint_proof_with_nulls_and_nonnullable_mix --nocapture`.
- Nullable suite also green: `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable`.

If workflows are gated because this is from a fork, please approve them. Happy to extend to more nullable types/expressions based on review.

Nicholas Toledo
