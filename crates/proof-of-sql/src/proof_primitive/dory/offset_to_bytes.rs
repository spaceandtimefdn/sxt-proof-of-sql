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
    fn we_can_convert_unsigned_values_to_little_endian_bytes() {
        assert_eq!(0u8.offset_to_bytes(), [0]);
        assert_eq!(u8::MAX.offset_to_bytes(), [u8::MAX]);
        assert_eq!(
            0x0102_0304_0506_0708_u64.offset_to_bytes(),
            [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn we_can_convert_boolean_values_to_bytes() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
    }

    #[test]
    fn we_can_offset_signed_integer_bounds_to_unsigned_bytes() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(0i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [u8::MAX]);

        assert_eq!(i16::MIN.offset_to_bytes(), 0u16.to_le_bytes());
        assert_eq!(0i16.offset_to_bytes(), (1u16 << 15).to_le_bytes());
        assert_eq!(i16::MAX.offset_to_bytes(), u16::MAX.to_le_bytes());

        assert_eq!(i32::MIN.offset_to_bytes(), 0u32.to_le_bytes());
        assert_eq!(0i32.offset_to_bytes(), (1u32 << 31).to_le_bytes());
        assert_eq!(i32::MAX.offset_to_bytes(), u32::MAX.to_le_bytes());

        assert_eq!(i64::MIN.offset_to_bytes(), 0u64.to_le_bytes());
        assert_eq!(0i64.offset_to_bytes(), (1u64 << 63).to_le_bytes());
        assert_eq!(i64::MAX.offset_to_bytes(), u64::MAX.to_le_bytes());

        assert_eq!(i128::MIN.offset_to_bytes(), 0u128.to_le_bytes());
        assert_eq!(0i128.offset_to_bytes(), (1u128 << 127).to_le_bytes());
        assert_eq!(i128::MAX.offset_to_bytes(), u128::MAX.to_le_bytes());
    }

    #[test]
    fn we_can_convert_scalar_limbs_to_bytes() {
        let limbs = [
            0x0102_0304_0506_0708,
            0x1112_1314_1516_1718,
            0x2122_2324_2526_2728,
            0x3132_3334_3536_3738,
        ];

        assert_eq!(
            limbs.offset_to_bytes(),
            [
                0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13,
                0x12, 0x11, 0x28, 0x27, 0x26, 0x25, 0x24, 0x23, 0x22, 0x21, 0x38, 0x37, 0x36, 0x35,
                0x34, 0x33, 0x32, 0x31,
            ]
        );
    }
}
