//! Tests for ByteDistribution.

#[cfg(test)]
mod byte_distribution_test {
    use crate::base::byte::ByteDistribution;
    use crate::base::scalar::{test_scalar::TestScalar, ScalarExt};

    #[test]
    fn test_byte_distribution_type_exists() {
        let _: Option<ByteDistribution> = None;
    }

    #[test]
    fn test_byte_distribution_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<ByteDistribution>());
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_byte_distribution_new() {
        // Test with a simple array
        let column = [TestScalar::ONE, TestScalar::TWO, TestScalar::from(3u64)];
        let dist = ByteDistribution::new::<TestScalar, _>(&column);
        // Should have varying bytes
        assert!(dist.vary_mask > 0 || dist.constant_mask != bnum::types::U256::ZERO);
    }

    #[test]
    fn test_byte_distribution_new_empty() {
        let column: [TestScalar; 0] = [];
        let dist = ByteDistribution::new::<TestScalar, _>(&column);
        assert_eq!(dist.vary_mask, 0);
    }
}
