/// Helper utilities for `QueryProof` unit tests.
///
/// This module is compiled only in `#[cfg(test)]` contexts and provides
/// convenience assertions that reduce boilerplate in individual test cases.
#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::{
        base::{
            commitment::naive_commitment::NaiveCommitment,
            database::{owned_table_utility::*, OwnedTable, OwnedTableTestAccessor},
            scalar::Curve25519Scalar,
        },
        sql::proof::{ProofPlan, QueryProof},
    };

    /// Verify that a proof for the given `plan` and `accessor` verifies
    /// successfully and that the resulting table matches `expected`.
    pub(crate) fn assert_proof_verifies<P>(
        plan: &P,
        accessor: &OwnedTableTestAccessor<NaiveCommitment>,
        expected: &OwnedTable<Curve25519Scalar>,
    ) where
        P: ProofPlan,
    {
        let (proof, result_expr) = QueryProof::<NaiveCommitment>::new(plan, accessor, &());
        let result = proof
            .verify(plan, accessor, &result_expr, &())
            .expect("proof should verify");
        assert_eq!(&result.table, expected, "verified table does not match expected");
    }

    /// Verify that a proof for the given `plan` and `accessor` fails to verify.
    pub(crate) fn assert_proof_fails<P>(
        plan: &P,
        accessor: &OwnedTableTestAccessor<NaiveCommitment>,
    ) where
        P: ProofPlan,
    {
        let (proof, result_expr) = QueryProof::<NaiveCommitment>::new(plan, accessor, &());
        assert!(
            proof.verify(plan, accessor, &result_expr, &()).is_err(),
            "proof verification should have failed"
        );
    }

    // ------------------------------------------------------------------
    // Self-tests for the helpers above
    // ------------------------------------------------------------------

    #[test]
    fn test_assert_proof_verifies_with_empty_table() {
        use crate::sql::proof::test_utility::EmptyTestQueryExpr;

        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let plan = EmptyTestQueryExpr {
            length: 0,
            ..Default::default()
        };
        let expected: OwnedTable<Curve25519Scalar> = owned_table([]);
        assert_proof_verifies(&plan, &accessor, &expected);
    }
}
