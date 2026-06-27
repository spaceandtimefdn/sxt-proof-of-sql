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
    fn unsigned_values_are_encoded_as_little_endian_bytes() {
        assert_eq!(0_u8.offset_to_bytes(), [0]);
        assert_eq!(255_u8.offset_to_bytes(), [255]);

        let value = 0x0123_4567_89ab_cdef_u64;
        assert_eq!(value.offset_to_bytes(), value.to_le_bytes());
    }

    #[test]
    fn signed_values_are_offset_before_little_endian_encoding() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), 0_u16.to_le_bytes());
        assert_eq!(0_i16.offset_to_bytes(), 0x8000_u16.to_le_bytes());
        assert_eq!(i16::MAX.offset_to_bytes(), u16::MAX.to_le_bytes());

        assert_eq!(i32::MIN.offset_to_bytes(), 0_u32.to_le_bytes());
        assert_eq!(0_i32.offset_to_bytes(), 0x8000_0000_u32.to_le_bytes());
        assert_eq!(i32::MAX.offset_to_bytes(), u32::MAX.to_le_bytes());

        assert_eq!(i64::MIN.offset_to_bytes(), 0_u64.to_le_bytes());
        assert_eq!(
            0_i64.offset_to_bytes(),
            0x8000_0000_0000_0000_u64.to_le_bytes()
        );
        assert_eq!(i64::MAX.offset_to_bytes(), u64::MAX.to_le_bytes());

        assert_eq!(i128::MIN.offset_to_bytes(), 0_u128.to_le_bytes());
        assert_eq!(
            0_i128.offset_to_bytes(),
            0x8000_0000_0000_0000_0000_0000_0000_0000_u128.to_le_bytes()
        );
        assert_eq!(i128::MAX.offset_to_bytes(), u128::MAX.to_le_bytes());
    }

    #[test]
    fn booleans_are_encoded_as_zero_or_one() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
    }

    #[test]
    fn u64_arrays_are_encoded_as_contiguous_native_endian_words() {
        let values = [0_u64, 1_u64, 0x0123_4567_89ab_cdef_u64, u64::MAX];
        let mut expected = [0; 32];
        for (index, value) in values.iter().enumerate() {
            let start = index * 8;
            expected[start..start + 8].copy_from_slice(&value.to_ne_bytes());
        }

        assert_eq!(values.offset_to_bytes(), expected);
    }
}
