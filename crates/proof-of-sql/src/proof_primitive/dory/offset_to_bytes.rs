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
    fn signed_offsets_map_min_zero_and_max_edges() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!((-1_i8).offset_to_bytes(), [127]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
        assert_eq!((-1_i16).offset_to_bytes(), 0x7fff_u16.to_le_bytes());
        assert_eq!(0_i16.offset_to_bytes(), 0x8000_u16.to_le_bytes());
        assert_eq!(i16::MAX.offset_to_bytes(), [255, 255]);

        assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
        assert_eq!((-1_i32).offset_to_bytes(), 0x7fff_ffff_u32.to_le_bytes());
        assert_eq!(0_i32.offset_to_bytes(), 0x8000_0000_u32.to_le_bytes());
        assert_eq!(i32::MAX.offset_to_bytes(), [255, 255, 255, 255]);
    }

    #[test]
    fn wide_signed_offsets_use_the_same_biasing_scheme() {
        assert_eq!(i64::MIN.offset_to_bytes(), [0; 8]);
        assert_eq!(
            0_i64.offset_to_bytes(),
            0x8000_0000_0000_0000_u64.to_le_bytes()
        );
        assert_eq!(i64::MAX.offset_to_bytes(), [255; 8]);

        assert_eq!(i128::MIN.offset_to_bytes(), [0; 16]);
        assert_eq!(
            0_i128.offset_to_bytes(),
            0x8000_0000_0000_0000_0000_0000_0000_0000_u128.to_le_bytes()
        );
        assert_eq!(i128::MAX.offset_to_bytes(), [255; 16]);
    }

    #[test]
    fn unsigned_and_bool_offsets_are_stored_without_bias() {
        assert_eq!(0_u8.offset_to_bytes(), [0]);
        assert_eq!(u8::MAX.offset_to_bytes(), [255]);

        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);

        assert_eq!(0_u64.offset_to_bytes(), [0; 8]);
        assert_eq!(
            0x0102_0304_0506_0708_u64.offset_to_bytes(),
            [8, 7, 6, 5, 4, 3, 2, 1]
        );
    }

    #[test]
    fn u64_array_offsets_are_contiguous_little_endian_limbs() {
        let limbs = [0_u64, 1, 0x0102_0304_0506_0708, u64::MAX];

        assert_eq!(
            limbs.offset_to_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, //
                1, 0, 0, 0, 0, 0, 0, 0, //
                8, 7, 6, 5, 4, 3, 2, 1, //
                255, 255, 255, 255, 255, 255, 255, 255,
            ]
        );
    }
}
