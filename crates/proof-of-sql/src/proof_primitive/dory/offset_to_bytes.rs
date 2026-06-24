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
    fn u8_returns_self_as_single_byte() {
        assert_eq!(42u8.offset_to_bytes(), [42u8]);
    }

    #[test]
    fn i8_min_maps_to_zero() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0u8]);
    }

    #[test]
    fn i8_zero_maps_to_128() {
        assert_eq!(0i8.offset_to_bytes(), [128u8]);
    }

    #[test]
    fn i8_max_maps_to_255() {
        assert_eq!(i8::MAX.offset_to_bytes(), [255u8]);
    }

    #[test]
    fn i16_min_maps_to_all_zeros() {
        assert_eq!(i16::MIN.offset_to_bytes(), [0u8, 0u8]);
    }

    #[test]
    fn i32_min_maps_to_all_zeros() {
        assert_eq!(i32::MIN.offset_to_bytes(), [0u8, 0u8, 0u8, 0u8]);
    }

    #[test]
    fn i64_min_maps_to_all_zeros() {
        assert_eq!(i64::MIN.offset_to_bytes(), [0u8; 8]);
    }

    #[test]
    fn i128_min_maps_to_all_zeros() {
        assert_eq!(i128::MIN.offset_to_bytes(), [0u8; 16]);
    }

    #[test]
    fn bool_false_maps_to_zero() {
        assert_eq!(false.offset_to_bytes(), [0u8]);
    }

    #[test]
    fn bool_true_maps_to_one() {
        assert_eq!(true.offset_to_bytes(), [1u8]);
    }

    #[test]
    fn u64_zero_maps_to_all_zeros() {
        assert_eq!(0u64.offset_to_bytes(), [0u8; 8]);
    }

    #[test]
    fn u64_one_maps_to_little_endian_one() {
        let bytes = 1u64.offset_to_bytes();
        assert_eq!(bytes[0], 1);
        assert_eq!(&bytes[1..], &[0u8; 7]);
    }
}
