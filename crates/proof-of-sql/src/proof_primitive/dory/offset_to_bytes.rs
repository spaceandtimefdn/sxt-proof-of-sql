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
    fn we_can_convert_unsigned_and_bool_offsets_to_bytes() {
        assert_eq!(0_u8.offset_to_bytes(), [0]);
        assert_eq!(255_u8.offset_to_bytes(), [255]);
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);

        let value = 0x0102_0304_0506_0708_u64;
        assert_eq!(value.offset_to_bytes(), [8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn we_can_convert_signed_offsets_to_order_preserving_bytes() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
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
    fn we_can_convert_u64_limbs_to_bytes() {
        let limbs = [
            0x0102_0304_0506_0708_u64,
            0x1112_1314_1516_1718,
            0x2122_2324_2526_2728,
            0x3132_3334_3536_3738,
        ];

        assert_eq!(
            limbs.offset_to_bytes(),
            [
                8, 7, 6, 5, 4, 3, 2, 1, 24, 23, 22, 21, 20, 19, 18, 17, 40, 39, 38, 37, 36, 35, 34,
                33, 56, 55, 54, 53, 52, 51, 50, 49,
            ]
        );
    }
}
