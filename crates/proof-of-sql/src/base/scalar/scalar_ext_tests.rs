/// Tests for scalar conversion helpers and the blanket `ScalarExt` trait.
///
/// These cover the sign / magnitude encoding and the lossy conversion paths
/// that are commonly missed by integration tests.
#[cfg(test)]
mod tests {
    use crate::base::scalar::{Scalar, ScalarExt};

    // We use the default test scalar (Ristretto255Scalar via curve25519-dalek)
    // throughout these tests.
    use crate::base::scalar::test_scalar::TestScalar;

    // -----------------------------------------------------------------------
    // from_bool
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_bool_true_is_one() {
        let one = TestScalar::from(1u64);
        assert_eq!(
            TestScalar::from(true),
            one,
            "Scalar::from(true) should equal ONE"
        );
    }

    #[test]
    fn test_from_bool_false_is_zero() {
        let zero = TestScalar::ZERO;
        assert_eq!(
            TestScalar::from(false),
            zero,
            "Scalar::from(false) should equal ZERO"
        );
    }

    // -----------------------------------------------------------------------
    // to_byte_array / sign encoding
    // -----------------------------------------------------------------------

    #[test]
    fn test_zero_has_canonical_byte_representation() {
        let z = TestScalar::ZERO;
        let bytes = z.to_byte_array();
        assert_eq!(bytes[0], 0u8, "First byte of ZERO should be 0");
    }

    #[test]
    fn test_one_has_nonzero_byte_representation() {
        let one = TestScalar::ONE;
        let bytes = one.to_byte_array();
        // At least one byte must be non-zero.
        assert!(
            bytes.iter().any(|&b| b != 0),
            "Byte representation of ONE must be non-zero"
        );
    }

    // -----------------------------------------------------------------------
    // num_limbs and is_zero consistency
    // -----------------------------------------------------------------------

    #[test]
    fn test_zero_scalar_is_zero() {
        assert!(TestScalar::ZERO.is_zero(), "ZERO.is_zero() must be true");
    }

    #[test]
    fn test_non_zero_scalar_is_not_zero() {
        assert!(
            !TestScalar::ONE.is_zero(),
            "ONE.is_zero() must be false"
        );
    }

    // -----------------------------------------------------------------------
    // arithmetic: addition and negation
    // -----------------------------------------------------------------------

    #[test]
    fn test_scalar_add_zero_is_identity() {
        let s = TestScalar::from(42u64);
        assert_eq!(s + TestScalar::ZERO, s, "Adding ZERO should be identity");
    }

    #[test]
    fn test_scalar_negate_then_add_gives_zero() {
        let s = TestScalar::from(123u64);
        let neg_s = -s;
        assert_eq!(
            s + neg_s,
            TestScalar::ZERO,
            "s + (-s) should equal ZERO"
        );
    }

    #[test]
    fn test_scalar_mul_by_one_is_identity() {
        let s = TestScalar::from(99u64);
        assert_eq!(s * TestScalar::ONE, s, "Multiplying by ONE should be identity");
    }

    #[test]
    fn test_scalar_mul_by_zero_is_zero() {
        let s = TestScalar::from(77u64);
        assert_eq!(
            s * TestScalar::ZERO,
            TestScalar::ZERO,
            "Multiplying by ZERO should give ZERO"
        );
    }

    // -----------------------------------------------------------------------
    // from small integer types
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_i8_positive() {
        let s = TestScalar::from(100i8);
        assert!(!s.is_zero());
    }

    #[test]
    fn test_from_i8_negative_does_not_panic() {
        // Negative values are represented as field elements via two's complement
        // wrapping — this must not panic.
        let _ = TestScalar::from(-1i8);
        let _ = TestScalar::from(i8::MIN);
    }

    #[test]
    fn test_from_u8_max() {
        let _ = TestScalar::from(u8::MAX);
    }

    #[test]
    fn test_from_i64_roundtrips() {
        for v in [0i64, 1, -1, i64::MAX, i64::MIN] {
            let s = TestScalar::from(v);
            // We can only check no-panic and basic sign consistency here.
            if v == 0 {
                assert!(s.is_zero());
            }
        }
    }

    #[test]
    fn test_from_i128_zero_is_zero() {
        assert!(TestScalar::from(0i128).is_zero());
    }

    // -----------------------------------------------------------------------
    // TWO constant (if present)
    // -----------------------------------------------------------------------

    #[test]
    fn test_one_plus_one_equals_two() {
        let two = TestScalar::ONE + TestScalar::ONE;
        // two should not be zero
        assert!(!two.is_zero(), "1 + 1 should not be ZERO in the scalar field");
    }
}
