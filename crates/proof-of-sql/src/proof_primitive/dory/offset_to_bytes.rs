pub trait OffsetToBytes<const LEN: usize> {
    fn offset_to_bytes(&self) -> [u8; LEN];
}

impl OffsetToBytes<1> for u8 {
    fn offset_to_bytes(&self) -> [u8; 1] {
        [*self]
    }
}

impl OffsetToBytes<1> for i8 {
    fn offset_to_bytes(&self) -> [u8; 1] {
        let shifted = self.wrapping_sub(i8::MIN);
        shifted.to_le_bytes()
    }
}

impl OffsetToBytes<2> for i16 {
    fn offset_to_bytes(&self) -> [u8; 2] {
        let shifted = self.wrapping_sub(i16::MIN);
        shifted.to_le_bytes()
    }
}

impl OffsetToBytes<4> for i32 {
    fn offset_to_bytes(&self) -> [u8; 4] {
        let shifted = self.wrapping_sub(i32::MIN);
        shifted.to_le_bytes()
    }
}

impl OffsetToBytes<8> for i64 {
    fn offset_to_bytes(&self) -> [u8; 8] {
        let shifted = self.wrapping_sub(i64::MIN);
        shifted.to_le_bytes()
    }
}

impl OffsetToBytes<16> for i128 {
    fn offset_to_bytes(&self) -> [u8; 16] {
        let shifted = self.wrapping_sub(i128::MIN);
        shifted.to_le_bytes()
    }
}

impl OffsetToBytes<1> for bool {
    fn offset_to_bytes(&self) -> [u8; 1] {
        [u8::from(*self)]
    }
}

impl OffsetToBytes<8> for u64 {
    fn offset_to_bytes(&self) -> [u8; 8] {
        self.to_le_bytes()
    }
}

impl OffsetToBytes<32> for [u64; 4] {
    fn offset_to_bytes(&self) -> [u8; 32] {
        bytemuck::cast(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::OffsetToBytes;

    #[test]
    fn u8_zero_maps_to_zero_byte() {
        assert_eq!(0u8.offset_to_bytes(), [0]);
    }

    #[test]
    fn u8_max_maps_to_255() {
        assert_eq!(255u8.offset_to_bytes(), [255]);
    }

    #[test]
    fn u8_midpoint_round_trips() {
        assert_eq!(128u8.offset_to_bytes(), [128]);
    }

    #[test]
    fn i8_min_maps_to_zero() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
    }

    #[test]
    fn i8_zero_maps_to_128() {
        assert_eq!(0i8.offset_to_bytes(), [128]);
    }

    #[test]
    fn i8_max_maps_to_255() {
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);
    }

    #[test]
    fn i8_minus_one_maps_to_127() {
        assert_eq!((-1i8).offset_to_bytes(), [127]);
    }

    #[test]
    fn i8_preserves_sort_order() {
        let a = (-100i8).offset_to_bytes()[0];
        let b = 0i8.offset_to_bytes()[0];
        let c = 100i8.offset_to_bytes()[0];
        assert!(a < b && b < c);
    }

    #[test]
    fn i16_min_maps_to_zero() {
        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
    }

    #[test]
    fn i16_zero_maps_to_midpoint() {
        let expected = 32768u16.to_le_bytes();
        assert_eq!(0i16.offset_to_bytes(), expected);
    }

    #[test]
    fn i16_max_maps_to_all_ones() {
        assert_eq!(i16::MAX.offset_to_bytes(), [0xFF, 0xFF]);
    }

    #[test]
    fn i16_minus_one_maps_correctly() {
        let expected = 32767u16.to_le_bytes();
        assert_eq!((-1i16).offset_to_bytes(), expected);
    }

    #[test]
    fn i32_min_maps_to_zero() {
        assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
    }

    #[test]
    fn i32_zero_maps_to_midpoint() {
        let expected = 2_147_483_648u32.to_le_bytes();
        assert_eq!(0i32.offset_to_bytes(), expected);
    }

    #[test]
    fn i32_max_maps_to_all_ones() {
        assert_eq!(i32::MAX.offset_to_bytes(), [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn i32_preserves_sort_order() {
        let a = i32::from_le_bytes(i32::MIN.offset_to_bytes());
        let b = i32::from_le_bytes(0i32.offset_to_bytes());
        let c = i32::from_le_bytes(i32::MAX.offset_to_bytes());
        assert!(a < b && b < c);
    }

    #[test]
    fn i64_min_maps_to_zero() {
        assert_eq!(i64::MIN.offset_to_bytes(), [0u8; 8]);
    }

    #[test]
    fn i64_zero_maps_to_midpoint() {
        let expected = (i64::MAX as u64 + 1).to_le_bytes();
        assert_eq!(0i64.offset_to_bytes(), expected);
    }

    #[test]
    fn i64_max_maps_to_all_ones() {
        assert_eq!(i64::MAX.offset_to_bytes(), [0xFF; 8]);
    }

    #[test]
    fn i64_minus_one_maps_correctly() {
        let expected = i64::MAX.unsigned_abs().to_le_bytes();
        assert_eq!((-1i64).offset_to_bytes(), expected);
    }

    #[test]
    fn i128_min_maps_to_zero() {
        assert_eq!(i128::MIN.offset_to_bytes(), [0u8; 16]);
    }

    #[test]
    fn i128_zero_maps_to_midpoint() {
        let expected = (i128::MAX as u128 + 1).to_le_bytes();
        assert_eq!(0i128.offset_to_bytes(), expected);
    }

    #[test]
    fn i128_max_maps_to_all_ones() {
        assert_eq!(i128::MAX.offset_to_bytes(), [0xFF; 16]);
    }

    #[test]
    fn bool_false_maps_to_zero() {
        assert_eq!(false.offset_to_bytes(), [0]);
    }

    #[test]
    fn bool_true_maps_to_one() {
        assert_eq!(true.offset_to_bytes(), [1]);
    }

    #[test]
    fn u64_zero_maps_to_zero_bytes() {
        assert_eq!(0u64.offset_to_bytes(), [0u8; 8]);
    }

    #[test]
    fn u64_max_maps_to_all_ones() {
        assert_eq!(u64::MAX.offset_to_bytes(), [0xFF; 8]);
    }

    #[test]
    fn u64_uses_little_endian_byte_order() {
        assert_eq!(1u64.offset_to_bytes(), [1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn u64_known_value_encodes_correctly() {
        let val: u64 = 0x0102_0304_0506_0708;
        assert_eq!(val.offset_to_bytes(), [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn u64_array_all_zero_maps_to_32_zero_bytes() {
        let arr: [u64; 4] = [0, 0, 0, 0];
        assert_eq!(arr.offset_to_bytes(), [0u8; 32]);
    }

    #[test]
    fn u64_array_first_element_affects_first_bytes() {
        let arr: [u64; 4] = [1, 0, 0, 0];
        let bytes = arr.offset_to_bytes();
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[1..8], [0u8; 7]);
        assert_eq!(bytes[8..32], [0u8; 24]);
    }

    #[test]
    fn u64_array_last_element_affects_last_bytes() {
        let arr: [u64; 4] = [0, 0, 0, 1];
        let bytes = arr.offset_to_bytes();
        assert_eq!(bytes[0..24], [0u8; 24]);
        assert_eq!(bytes[24], 1);
        assert_eq!(bytes[25..32], [0u8; 7]);
    }
}
