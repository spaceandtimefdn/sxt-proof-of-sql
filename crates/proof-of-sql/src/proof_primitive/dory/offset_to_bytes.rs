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
    fn we_offset_signed_integers_before_serializing() {
        assert_eq!(i8::MIN.offset_to_bytes(), [0]);
        assert_eq!(0_i8.offset_to_bytes(), [128]);
        assert_eq!(i8::MAX.offset_to_bytes(), [255]);

        assert_eq!(i16::MIN.offset_to_bytes(), 0_u16.to_le_bytes());
        assert_eq!(0_i16.offset_to_bytes(), 32768_u16.to_le_bytes());
        assert_eq!(i16::MAX.offset_to_bytes(), u16::MAX.to_le_bytes());
    }

    #[test]
    fn we_serialize_unsigned_bool_and_limb_arrays_without_offsets() {
        assert_eq!(false.offset_to_bytes(), [0]);
        assert_eq!(true.offset_to_bytes(), [1]);
        assert_eq!(7_u8.offset_to_bytes(), [7]);
        assert_eq!(11_u64.offset_to_bytes(), 11_u64.to_le_bytes());

        let limbs = [1_u64, 2, 3, 4];
        let mut expected = [0_u8; 32];
        for (index, limb) in limbs.iter().enumerate() {
            expected[index * 8..(index + 1) * 8].copy_from_slice(&limb.to_le_bytes());
        }
        assert_eq!(limbs.offset_to_bytes(), expected);
    }
}
