#[cfg(test)]
mod tests {
    use super::super::verify_evaluate_filter;
    use crate::base::{
        proof::{ProofError, ProofSizeMismatch},
        scalar::test_scalar::TestScalar,
    };
    use crate::sql::proof::mock_verification_builder::MockVerificationBuilder;

    // --- verify_evaluate_filter happy path ---

    #[test]
    fn we_can_verify_evaluate_filter_with_zero_evaluations() {
        // When final_round_mles has no rows, try_consume_final_round_mle_evaluation
        // defaults to S::ZERO. With max_multiplicands = 3, all four subpolynomial
        // evaluations (three Identity degree-2 and one ZeroSum degree-2) succeed.
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            3,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let result = verify_evaluate_filter(
            &mut builder,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn we_can_verify_evaluate_filter_with_explicit_mle_values() {
        // Provide two final-round MLE evaluations for c_star and d_star.
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(5u64), TestScalar::from(7u64)]],
            vec![],
            vec![],
            vec![],
        );
        let result = verify_evaluate_filter(
            &mut builder,
            TestScalar::ONE,
            TestScalar::ONE,
            TestScalar::ONE,
            TestScalar::ONE,
            TestScalar::ONE,
        );
        assert!(result.is_ok());
    }

    // --- verify_evaluate_filter error paths ---

    #[test]
    fn we_get_error_when_second_final_round_mle_evaluation_is_missing() {
        // Row 0 has only one MLE value. The second try_consume_final_round_mle_evaluation
        // (for d_star_eval) runs out of entries → TooFewMLEEvaluations.
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::ONE]], // only c_star available, d_star missing
            vec![],
            vec![],
            vec![],
        );
        let err = verify_evaluate_filter(
            &mut builder,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofError::ProofSizeMismatch {
                source: ProofSizeMismatch::TooFewMLEEvaluations
            }
        ));
    }

    #[test]
    fn we_get_error_when_identity_subpolynomial_degree_exceeds_max_multiplicands() {
        // subpolynomial_max_multiplicands = 2, but Identity subpolynomials have degree 2
        // which requires degree + 1 = 3 <= max_multiplicands. With max = 2 → error.
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            2,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let err = verify_evaluate_filter(
            &mut builder,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ZERO,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofError::ProofSizeMismatch {
                source: ProofSizeMismatch::SumcheckProofTooSmall
            }
        ));
    }
}
