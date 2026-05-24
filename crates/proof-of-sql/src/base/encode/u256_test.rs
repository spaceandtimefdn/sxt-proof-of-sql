//! Tests for U256 type.

#[cfg(test)]
mod u256_test {
    use crate::base::encode::u256::U256;

    #[test]
    fn test_u256_from_words() {
        let val = U256::from_words(0x12345678_90ABCDEFu128, 0xFEDCBA98_76543210u128);
        assert_eq!(val.low, 0x12345678_90ABCDEFu128);
        assert_eq!(val.high, 0xFEDCBA98_76543210u128);
    }

    #[test]
    fn test_u256_partial_eq() {
        let val1 = U256::from_words(1, 2);
        let val2 = U256::from_words(1, 2);
        let val3 = U256::from_words(2, 1);
        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_u256_debug() {
        let val = U256::from_words(1, 2);
        let debug_str = format!("{:?}", val);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_u256_clone() {
        let val = U256::from_words(123, 456);
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }
}
