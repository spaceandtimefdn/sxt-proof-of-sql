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
    fn unsigned_and_bool_values_use_direct_byte_representations() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
        assert_eq!(0xab_u8.offset_to_bytes(), [0xab]);
        assert_eq!(
            0x0102_0304_0506_0708_u64.offset_to_bytes(),
            [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn signed_values_are_offset_before_little_endian_encoding() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0x00]);
        assert_eq!(0_i8.offset_to_bytes(), [0x80]);
        assert_eq!(i8::MAX.offset_to_bytes(), [0xff]);

        assert_eq!(i16::MIN.offset_to_bytes(), [0x00, 0x00]);
        assert_eq!(0_i16.offset_to_bytes(), [0x00, 0x80]);
        assert_eq!(i16::MAX.offset_to_bytes(), [0xff, 0xff]);

        assert_eq!(i32::MIN.offset_to_bytes(), [0x00; 4]);
        assert_eq!(0_i32.offset_to_bytes(), [0x00, 0x00, 0x00, 0x80]);
        assert_eq!(i32::MAX.offset_to_bytes(), [0xff; 4]);

        assert_eq!(i64::MIN.offset_to_bytes(), [0x00; 8]);
        assert_eq!(
            0_i64.offset_to_bytes(),
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
        );
        assert_eq!(i64::MAX.offset_to_bytes(), [0xff; 8]);

        let mut zero_i128_bytes = [0x00; 16];
        zero_i128_bytes[15] = 0x80;
        assert_eq!(i128::MIN.offset_to_bytes(), [0x00; 16]);
        assert_eq!(0_i128.offset_to_bytes(), zero_i128_bytes);
        assert_eq!(i128::MAX.offset_to_bytes(), [0xff; 16]);
    }

    #[test]
    fn u64_array_offsets_keep_element_order_and_native_layout() {
        let values = [
            0x0102_0304_0506_0708_u64,
            0x1112_1314_1516_1718_u64,
            0x2122_2324_2526_2728_u64,
            0x3132_3334_3536_3738_u64,
        ];

        let bytes = values.offset_to_bytes();

        assert_eq!(&bytes[0..8], &values[0].to_ne_bytes());
        assert_eq!(&bytes[8..16], &values[1].to_ne_bytes());
        assert_eq!(&bytes[16..24], &values[2].to_ne_bytes());
        assert_eq!(&bytes[24..32], &values[3].to_ne_bytes());
    }
}
