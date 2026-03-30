/// Tests for commitment evaluation proof helper traits and blanket
/// implementations that are not exercised by the main proving pipeline tests.
#[cfg(test)]
mod tests {
    use crate::base::commitment::CommitmentEvaluationProof;
    use crate::proof_primitive::dory::DoryEvaluationProof;

    // We only check the *shape* of the API — that the associated types can be
    // named and the trait object is Sized where expected.  Full round-trip tests
    // live in the integration test suite; this module targets uncovered helpers.

    /// Ensure that `DoryEvaluationProof` implements `CommitmentEvaluationProof`
    /// (trait-bound satisfaction check, caught at compile time).
    #[test]
    fn test_dory_evaluation_proof_implements_trait() {
        fn assert_impl<T: CommitmentEvaluationProof>() {}
        assert_impl::<DoryEvaluationProof>();
    }
}
