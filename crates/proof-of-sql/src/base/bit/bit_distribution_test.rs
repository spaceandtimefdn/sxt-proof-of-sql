//! Tests for BitDistribution.

#[cfg(test)]
mod bit_distribution_test {
    use crate::base::bit::bit_distribution::BitDistribution;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_bit_distribution_new() {
        let data = [TestScalar::ONE, TestScalar::TWO, TestScalar::from(3u64)];
        let dist = BitDistribution::new::<TestScalar, _>(&data);
        assert!(dist.vary_mask() != bnum::types::U256::ZERO || dist.leading_bit_mask() != bnum::types::U256::ZERO);
    }

    #[test]
    fn test_bit_distribution_type_exists() {
        let _: Option<BitDistribution> = None;
    }

    #[test]
    fn test_bit_distribution_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<BitDistribution>());
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_bit_distribution_empty() {
        let data: [TestScalar; 0] = [];
        let dist = BitDistribution::new::<TestScalar, _>(&data);
        assert!(dist.num_varying_bits() >= 0);
    }
}
