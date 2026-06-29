//! Tests for log module.

#[cfg(test)]
mod log_test {
    use crate::base::math::log::log2_up;

    #[test]
    fn test_log2_up_basic() {
        // Test power of 2
        assert_eq!(log2_up(1), 0);
        assert_eq!(log2_up(2), 1);
        assert_eq!(log2_up(4), 2);
        assert_eq!(log2_up(8), 3);
    }

    #[test]
    fn test_log2_up_non_power_of_two() {
        // Test non-power of 2
        assert_eq!(log2_up(3), 2);  // ceil(log2(3)) = 2
        assert_eq!(log2_up(5), 3);  // ceil(log2(5)) = 3
        assert_eq!(log2_up(9), 4);  // ceil(log2(9)) = 4
    }
}
