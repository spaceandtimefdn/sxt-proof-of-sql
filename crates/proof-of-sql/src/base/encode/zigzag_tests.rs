/// Tests for zigzag encoding used in proof serialization
#[cfg(test)]
mod tests {
    use crate::base::encode::{zigzag_decode, zigzag_encode};

    #[test]
    fn test_zigzag_encode_zero() {
        assert_eq!(zigzag_encode(0i64), 0u64);
    }

    #[test]
    fn test_zigzag_encode_positive() {
        assert_eq!(zigzag_encode(1i64), 2u64);
        assert_eq!(zigzag_encode(2i64), 4u64);
        assert_eq!(zigzag_encode(i64::MAX), u64::MAX - 1);
    }

    #[test]
    fn test_zigzag_encode_negative() {
        assert_eq!(zigzag_encode(-1i64), 1u64);
        assert_eq!(zigzag_encode(-2i64), 3u64);
        assert_eq!(zigzag_encode(i64::MIN), u64::MAX);
    }

    #[test]
    fn test_zigzag_roundtrip() {
        for val in [-1000i64, -1, 0, 1, 1000, i64::MAX, i64::MIN] {
            assert_eq!(zigzag_decode(zigzag_encode(val)), val);
        }
    }

    #[test]
    fn test_zigzag_decode_zero() {
        assert_eq!(zigzag_decode(0u64), 0i64);
    }

    #[test]
    fn test_zigzag_decode_one() {
        assert_eq!(zigzag_decode(1u64), -1i64);
    }

    #[test]
    fn test_zigzag_decode_two() {
        assert_eq!(zigzag_decode(2u64), 1i64);
    }
}
