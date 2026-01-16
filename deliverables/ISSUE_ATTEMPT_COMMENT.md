/attempt #183

Updated plan with the PoC hook:
- End-to-end nullable proof now runs without blitzar using NaiveEvaluationProof; mixes canonicalized nullable bigint data with a non-nullable column and filters nulls via validity.
- Command: `cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_bigint_proof_with_nulls_and_nonnullable_mix`
- PR: https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120

Nicholas Toledo / Toledo Technologies LLC
