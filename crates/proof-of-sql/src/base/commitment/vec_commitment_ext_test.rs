/// Tests for VecCommitmentExt helper methods
#[cfg(test)]
mod tests {
    use crate::base::commitment::{
        naive_evaluation_proof::NaiveEvaluationProof, vec_commitment_ext::VecCommitmentExt,
        CommitmentEvaluationProof,
    };
    use crate::base::scalar::{test_scalar::TestScalar, Scalar};

    // Use the test commitment type that ships with the crate.
    type TestCommit = crate::base::commitment::naive_evaluation_proof::NaiveCommitment;

    /// Extending an empty commitment vec by zero extra slots is a no-op.
    #[test]
    fn test_extend_with_zeros_from_empty() {
        let mut commits: Vec<TestCommit> = vec![];
        commits.extend_with_zero(0);
        assert!(commits.is_empty());
    }

    /// Appending zero commitments increases the length correctly.
    #[test]
    fn test_extend_with_zeros_adds_correct_count() {
        let mut commits: Vec<TestCommit> = vec![];
        commits.extend_with_zero(3);
        assert_eq!(commits.len(), 3);
    }

    /// Two separate zero extensions equal a single combined extension.
    #[test]
    fn test_extend_with_zeros_twice_equals_once() {
        let mut a: Vec<TestCommit> = vec![];
        a.extend_with_zero(2);
        a.extend_with_zero(3);

        let mut b: Vec<TestCommit> = vec![];
        b.extend_with_zero(5);

        assert_eq!(a, b);
    }
}
