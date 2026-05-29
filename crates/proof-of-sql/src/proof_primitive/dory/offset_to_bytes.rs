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
    fn signed_integers_are_offset_before_little_endian_encoding() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!((-1_i8).offset_to_bytes(), [127]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
        assert_eq!((-1_i16).offset_to_bytes(), [255, 127]);
        assert_eq!(0_i16.offset_to_bytes(), [0, 128]);
        assert_eq!(i16::MAX.offset_to_bytes(), [255, 255]);

        assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
        assert_eq!(0_i32.offset_to_bytes(), [0, 0, 0, 128]);
        assert_eq!(i32::MAX.offset_to_bytes(), [255, 255, 255, 255]);

        assert_eq!(i64::MIN.offset_to_bytes(), [0; 8]);
        assert_eq!(0_i64.offset_to_bytes(), [0, 0, 0, 0, 0, 0, 0, 128]);
        assert_eq!(i64::MAX.offset_to_bytes(), [255; 8]);

        assert_eq!(i128::MIN.offset_to_bytes(), [0; 16]);
        assert_eq!(
            0_i128.offset_to_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128]
        );
        assert_eq!(i128::MAX.offset_to_bytes(), [255; 16]);
    }

    #[test]
    fn unsigned_bool_and_word_array_values_use_their_raw_little_endian_bytes() {
        assert_eq!(255_u8.offset_to_bytes(), [255]);
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
        assert_eq!(
            0x0123_4567_89ab_cdef_u64.offset_to_bytes(),
            [0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01]
        );

        let words = [
            0x0123_4567_89ab_cdef_u64,
            0xfedc_ba98_7654_3210_u64,
            0x0000_0000_0000_0001_u64,
            0x8000_0000_0000_0000_u64,
        ];
        assert_eq!(
            words.offset_to_bytes(),
            [
                0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba,
                0xdc, 0xfe, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x80,
            ]
        );
    }
}
