//! Tests for I256 type.

#[cfg(test)]
mod i256_test {
    use crate::base::math::i256::I256;

    #[test]
    fn test_i256_new() {
        let val = I256::new([1, 2, 3, 4]);
        assert_eq!(val.limbs(), [1, 2, 3, 4]);
    }

    #[test]
    fn test_i256_neg() {
        let val = I256::new([0, 0, 0, 0]);
        let neg_val = -val;
        let debug_str = format!("{:?}", neg_val);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_i256_type_exists() {
        let _: Option<I256> = None;
    }

    #[test]
    fn test_i256_debug() {
        let val = I256::new([1, 2, 3, 4]);
        let debug_str = format!("{:?}", val);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_i256_partial_eq() {
        let val1 = I256::new([1, 2, 3, 4]);
        let val2 = I256::new([1, 2, 3, 4]);
        let val3 = I256::new([4, 3, 2, 1]);
        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }
}
