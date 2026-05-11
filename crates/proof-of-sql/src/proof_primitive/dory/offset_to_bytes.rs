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
    use super::*;

    #[test]
    fn unsigned_integer_offsets_are_little_endian() {
        assert_eq!(0_u8.offset_to_bytes(), [0]);
        assert_eq!(255_u8.offset_to_bytes(), [255]);

        let value = 0x0102_0304_0506_0708_u64;
        assert_eq!(value.offset_to_bytes(), value.to_le_bytes());
    }

    #[test]
    fn signed_integer_offsets_shift_minimum_to_zero() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
        assert_eq!(
            0_i16.offset_to_bytes(),
            (0_i16.wrapping_sub(i16::MIN)).to_le_bytes()
        );
        assert_eq!(i16::MAX.offset_to_bytes(), [255, 255]);

        assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
        assert_eq!(
            0_i32.offset_to_bytes(),
            (0_i32.wrapping_sub(i32::MIN)).to_le_bytes()
        );
        assert_eq!(i32::MAX.offset_to_bytes(), [255, 255, 255, 255]);

        assert_eq!(i64::MIN.offset_to_bytes(), [0; 8]);
        assert_eq!(
            0_i64.offset_to_bytes(),
            (0_i64.wrapping_sub(i64::MIN)).to_le_bytes()
        );
        assert_eq!(i64::MAX.offset_to_bytes(), [255; 8]);

        assert_eq!(i128::MIN.offset_to_bytes(), [0; 16]);
        assert_eq!(
            0_i128.offset_to_bytes(),
            (0_i128.wrapping_sub(i128::MIN)).to_le_bytes()
        );
        assert_eq!(i128::MAX.offset_to_bytes(), [255; 16]);
    }

    #[test]
    fn bool_offsets_match_boolean_value() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
    }

    #[test]
    fn u64_limb_arrays_keep_native_byte_layout() {
        let limbs = [1_u64, 2, u64::MAX, 0x0102_0304_0506_0708];
        let bytes = limbs.offset_to_bytes();
        let mut expected = [0_u8; 32];
        for (chunk, limb) in expected.chunks_exact_mut(8).zip(limbs) {
            chunk.copy_from_slice(&limb.to_ne_bytes());
        }
        assert_eq!(bytes, expected);
    }
}
