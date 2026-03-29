/// Tests for log utilities
#[cfg(test)]
mod tests {
    use crate::base::math::log::{log2_up, next_power_of_two_if_nonzero};

    #[test]
    fn test_log2_up_of_one() {
        assert_eq!(log2_up(1), 0);
    }

    #[test]
    fn test_log2_up_of_two() {
        assert_eq!(log2_up(2), 1);
    }

    #[test]
    fn test_log2_up_of_power_of_two() {
        assert_eq!(log2_up(8), 3);
        assert_eq!(log2_up(16), 4);
        assert_eq!(log2_up(1024), 10);
    }

    #[test]
    fn test_log2_up_non_power_of_two_rounds_up() {
        // log2(3) ~ 1.58, should round up to 2
        assert_eq!(log2_up(3), 2);
        // log2(5) ~ 2.32, should round up to 3
        assert_eq!(log2_up(5), 3);
        // log2(9) ~ 3.17, should round up to 4
        assert_eq!(log2_up(9), 4);
    }

    #[test]
    fn test_next_power_of_two_if_nonzero_of_zero() {
        assert_eq!(next_power_of_two_if_nonzero(0), 0);
    }

    #[test]
    fn test_next_power_of_two_if_nonzero_of_one() {
        assert_eq!(next_power_of_two_if_nonzero(1), 1);
    }

    #[test]
    fn test_next_power_of_two_if_nonzero_of_power_of_two() {
        assert_eq!(next_power_of_two_if_nonzero(4), 4);
        assert_eq!(next_power_of_two_if_nonzero(8), 8);
    }

    #[test]
    fn test_next_power_of_two_if_nonzero_rounds_up() {
        assert_eq!(next_power_of_two_if_nonzero(3), 4);
        assert_eq!(next_power_of_two_if_nonzero(5), 8);
        assert_eq!(next_power_of_two_if_nonzero(9), 16);
        assert_eq!(next_power_of_two_if_nonzero(100), 128);
    }
}
