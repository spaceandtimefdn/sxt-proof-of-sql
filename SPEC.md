# SPEC

## [spaceandtimefdn/sxt-proof-of-sql] test: improve test performance

Task id: `github:spaceandtimefdn/sxt-proof-of-sql#557`

## Goal
# Background and Motivation

Currently, running all the tests takes a long time: almost 15 min on my local machine.

# Changes Required

Reduce the test times, without removing or weakening any tests.

EDIT: More broadly, any improvement to the overall CI runtimes are welcome changes and will qualify for the bounty. This ultimately the main goal of this issue, with a secondary goal being faster local development work.

- [ ] The most obvious and easiest way to improve the performance is by caching the Dory setups. I believe (although have not confirmed) that `PublicParameters::test_rand`, `ProverSetup::from`, and `VerifierSetup::from` are taking up over half of the entire test suite time.
    - This could be done by either saving test setups to a file if they are not found, by simply committing test versions to the repo, or some other approach. I am open to creative solutions.


Below is a the output of `cargo nextest run -j1 --all-features | grep -v "\[   "`.

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
────────────
 Nextest run ID 045165c7-1b36-4116-94aa-fb64511d54e7 with nextest profile: default
    Starting 1506 tests across 7 binaries (2 tests skipped)
        PASS [  14.648s] proof-of-sql proof_primitive::dory::dory_commitment_evaluation_proof_test::test_random_ipa_with_length_1
        PASS [  57.824s] proof-of-sql proof_primitive::dory::dory_commitment_evaluation_proof_test::test_random_ipa_with_various_lengths
        PASS [  14.638s] proof-of-sql proof_primitive::dory::dory_commitment_evaluation_proof_test::test_simple_ipa
        PASS [  23.490s] proof-of-sql proof_primitive::dory::dynamic_dory_commitment_evaluation_proof_test::test_random_ipa_with_various_lengths
        PASS [  10.478s] proof-of-sql proof_primitive::dory::setup_test::we_can_create_prover_setups_with_various_sizes
        PASS [  10.288s] proof-of-sql sql::proof_exprs::inequality_expr_test::we_can_compare_columns_with_extreme_values
        PASS [  13.511s] proof-of-sql sql::proof_plans::sort_merge_join_exec_test::we_can_prove_and_get_the_correct_result_from_a_complex_query_involving_two_sort_merge_joins
────────────
     Summary [ 814.675s] 1506 tests run: 1506 passed, 2 skipped
```

## Acceptance criteria
- [ ] The most obvious and easiest way to improve the performance is by caching the Dory setups. I believe (although have not confirmed) that `PublicParameters::test_rand`, `ProverSetup::from`, and `VerifierSetup::from` are taking up over half of the entire test suite time.
- This could be done by either saving test setups to a file if they are not found, by simply committing test versions to the repo, or some other approach. I am open to creative solutions.

## Stack hints
(any)



Produce ALL files as a single JSON response. No markdown fences, no prose — just the JSON object.
