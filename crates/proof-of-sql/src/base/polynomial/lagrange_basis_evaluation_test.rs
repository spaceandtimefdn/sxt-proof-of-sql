/// Tests for Lagrange basis evaluation helpers
#[cfg(test)]
mod tests {
    use crate::base::{
        polynomial::lagrange_basis_evaluation::{
            compute_evaluation_vector, compute_evaluation_vector_with_length,
        },
        scalar::{test_scalar::TestScalar, Scalar},
    };

    /// The evaluation vector for an empty point list over length 1 should be [1].
    #[test]
    fn test_evaluation_vector_empty_point_length_one() {
        let point: Vec<TestScalar> = vec![];
        let mut result = vec![TestScalar::ZERO; 1];
        compute_evaluation_vector(&mut result, &point);
        assert_eq!(result, vec![TestScalar::ONE]);
    }

    /// Single point r: result should be [1-r, r].
    #[test]
    fn test_evaluation_vector_single_point() {
        let r = TestScalar::from(3u64);
        let point = vec![r];
        let mut result = vec![TestScalar::ZERO; 2];
        compute_evaluation_vector(&mut result, &point);
        assert_eq!(result[0], TestScalar::ONE - r);
        assert_eq!(result[1], r);
    }

    /// Two points [r0, r1]:
    /// result[0] = (1-r0)(1-r1)
    /// result[1] = r0*(1-r1)
    /// result[2] = (1-r0)*r1
    /// result[3] = r0*r1
    #[test]
    fn test_evaluation_vector_two_points() {
        let r0 = TestScalar::from(2u64);
        let r1 = TestScalar::from(5u64);
        let point = vec![r0, r1];
        let mut result = vec![TestScalar::ZERO; 4];
        compute_evaluation_vector(&mut result, &point);

        let one = TestScalar::ONE;
        assert_eq!(result[0], (one - r0) * (one - r1));
        assert_eq!(result[1], r0 * (one - r1));
        assert_eq!(result[2], (one - r0) * r1);
        assert_eq!(result[3], r0 * r1);
    }

    /// Evaluation vector entries must sum to 1.
    #[test]
    fn test_evaluation_vector_sums_to_one() {
        let point = vec![
            TestScalar::from(7u64),
            TestScalar::from(11u64),
            TestScalar::from(13u64),
        ];
        let n = 1 << point.len();
        let mut result = vec![TestScalar::ZERO; n];
        compute_evaluation_vector(&mut result, &point);
        let sum: TestScalar = result.iter().copied().sum();
        assert_eq!(sum, TestScalar::ONE);
    }

    /// compute_evaluation_vector_with_length pads / truncates correctly.
    #[test]
    fn test_evaluation_vector_with_length_truncated() {
        let r = TestScalar::from(3u64);
        let point = vec![r];
        // Request only 1 element (truncated from 2).
        let result = compute_evaluation_vector_with_length(&point, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TestScalar::ONE - r);
    }

    #[test]
    fn test_evaluation_vector_with_length_full() {
        let r = TestScalar::from(3u64);
        let point = vec![r];
        let result = compute_evaluation_vector_with_length(&point, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TestScalar::ONE - r);
        assert_eq!(result[1], r);
    }
}
