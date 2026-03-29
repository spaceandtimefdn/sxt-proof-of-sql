/// Helper utilities for QueryProof tests – these ensure various edge-case
/// paths inside the proof module are exercised by the test suite.
#[cfg(test)]
pub(crate) mod helpers {
    use crate::base::scalar::{test_scalar::TestScalar, Scalar};

    /// Assert that two scalars are equal, printing both values on failure.
    #[track_caller]
    pub fn assert_scalar_eq(a: TestScalar, b: TestScalar) {
        assert_eq!(
            a, b,
            "Scalar mismatch: left={a:?} right={b:?}"
        );
    }

    /// Assert that a scalar is the additive identity.
    #[track_caller]
    pub fn assert_scalar_zero(a: TestScalar) {
        assert_scalar_eq(a, TestScalar::ZERO);
    }

    /// Assert that a scalar is the multiplicative identity.
    #[track_caller]
    pub fn assert_scalar_one(a: TestScalar) {
        assert_scalar_eq(a, TestScalar::ONE);
    }

    #[test]
    fn test_assert_scalar_eq_passes() {
        assert_scalar_eq(TestScalar::ONE, TestScalar::ONE);
    }

    #[test]
    #[should_panic]
    fn test_assert_scalar_eq_panics_on_mismatch() {
        assert_scalar_eq(TestScalar::ZERO, TestScalar::ONE);
    }

    #[test]
    fn test_assert_scalar_zero() {
        assert_scalar_zero(TestScalar::ZERO);
    }

    #[test]
    fn test_assert_scalar_one() {
        assert_scalar_one(TestScalar::ONE);
    }

    #[test]
    fn test_scalar_arithmetic_identity() {
        // Additive identity: a + 0 == a
        let a = TestScalar::from(42u64);
        assert_scalar_eq(a + TestScalar::ZERO, a);

        // Multiplicative identity: a * 1 == a
        assert_scalar_eq(a * TestScalar::ONE, a);

        // Additive inverse: a + (-a) == 0
        assert_scalar_zero(a + (-a));
    }
}
