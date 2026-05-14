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
    fn signed_offsets_shift_min_to_zero() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
        assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
        assert_eq!(i64::MIN.offset_to_bytes(), [0; 8]);
        assert_eq!(i128::MIN.offset_to_bytes(), [0; 16]);
    }

    #[test]
    fn signed_offsets_shift_across_sign_boundary() {
        let negative_one = (-1_i32).offset_to_bytes();
        let zero = 0_i32.offset_to_bytes();
        let positive_one = 1_i32.offset_to_bytes();

        assert_eq!(negative_one, i32::MAX.to_le_bytes());
        assert_eq!(zero, i32::MIN.to_le_bytes());
        assert_eq!(positive_one, i32::MIN.wrapping_add(1).to_le_bytes());
    }

    #[test]
    fn bool_and_unsigned_offsets_use_direct_bytes() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
        assert_eq!(
            0x0123_4567_89ab_cdef_u64.offset_to_bytes(),
            [0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01,]
        );
    }

    #[test]
    fn u64_array_offsets_concatenate_words() {
        let words = [
            0x0123_4567_89ab_cdef_u64,
            0xfedc_ba98_7654_3210_u64,
            0,
            u64::MAX,
        ];

        assert_eq!(
            words.offset_to_bytes(),
            [
                0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba,
                0xdc, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff,
            ]
        );
    }
}
