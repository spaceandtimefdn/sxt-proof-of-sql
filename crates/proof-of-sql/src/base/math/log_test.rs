#[cfg(test)]
mod tests {
    use crate::base::math::log::log2_up;

    #[test]
    fn test_log2_up_of_zero() {
        assert_eq!(log2_up(0), 0);
    }

    #[test]
    fn test_log2_up_of_one() {
        assert_eq!(log2_up(1), 0);
    }

    #[test]
    fn test_log2_up_of_two() {
        assert_eq!(log2_up(2), 1);
    }

    #[test]
    fn test_log2_up_exact_powers_of_two() {
        assert_eq!(log2_up(4), 2);
        assert_eq!(log2_up(8), 3);
        assert_eq!(log2_up(16), 4);
        assert_eq!(log2_up(32), 5);
        assert_eq!(log2_up(64), 6);
        assert_eq!(log2_up(128), 7);
        assert_eq!(log2_up(256), 8);
    }

    #[test]
    fn test_log2_up_non_power_of_two_rounds_up() {
        // Values between powers of two should round up.
        assert_eq!(log2_up(3), 2);   // 2^1 < 3 <= 2^2
        assert_eq!(log2_up(5), 3);   // 2^2 < 5 <= 2^3
        assert_eq!(log2_up(6), 3);
        assert_eq!(log2_up(7), 3);
        assert_eq!(log2_up(9), 4);
        assert_eq!(log2_up(15), 4);
        assert_eq!(log2_up(17), 5);
        assert_eq!(log2_up(100), 7); // 2^6=64 < 100 <= 2^7=128
    }

    #[test]
    fn test_log2_up_large_value() {
        // 2^20 = 1_048_576
        assert_eq!(log2_up(1_048_576_usize), 20);
        assert_eq!(log2_up(1_048_577_usize), 21);
    }
}
