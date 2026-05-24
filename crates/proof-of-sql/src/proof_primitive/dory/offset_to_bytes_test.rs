//! Tests for OffsetToBytes trait.

#[cfg(test)]
mod offset_to_bytes_test {
    use crate::proof_primitive::dory::offset_to_bytes::OffsetToBytes;

    #[test]
    fn test_u8_offset_to_bytes() {
        let val: u8 = 42;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes, [42]);
    }

    #[test]
    fn test_i8_offset_to_bytes() {
        let val: i8 = -64;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 1);
    }

    #[test]
    fn test_i16_offset_to_bytes() {
        let val: i16 = 100;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn test_i32_offset_to_bytes() {
        let val: i32 = 12345;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_i64_offset_to_bytes() {
        let val: i64 = 9876543210;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn test_i128_offset_to_bytes() {
        let val: i128 = 12345678901234567890i128;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_bool_offset_to_bytes() {
        let true_val = true;
        let false_val = false;
        assert_eq!(true_val.offset_to_bytes(), [1]);
        assert_eq!(false_val.offset_to_bytes(), [0]);
    }

    #[test]
    fn test_u64_offset_to_bytes() {
        let val: u64 = 0x1234567890ABCDEFu64;
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn test_u64_array_offset_to_bytes() {
        let val: [u64; 4] = [1, 2, 3, 4];
        let bytes = val.offset_to_bytes();
        assert_eq!(bytes.len(), 32);
    }
}