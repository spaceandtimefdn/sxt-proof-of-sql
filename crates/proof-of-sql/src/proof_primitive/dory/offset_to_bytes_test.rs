use super::offset_to_bytes::OffsetToBytes;

#[test]
fn we_can_offset_signed_integer_bytes_at_boundaries() {
    assert_eq!(i8::MIN.offset_to_bytes(), [0]);
    assert_eq!(0_i8.offset_to_bytes(), [128]);
    assert_eq!(i8::MAX.offset_to_bytes(), [255]);

    assert_eq!(i16::MIN.offset_to_bytes(), 0_u16.to_le_bytes());
    assert_eq!(0_i16.offset_to_bytes(), 32768_u16.to_le_bytes());
    assert_eq!(i16::MAX.offset_to_bytes(), u16::MAX.to_le_bytes());

    assert_eq!(i32::MIN.offset_to_bytes(), 0_u32.to_le_bytes());
    assert_eq!(0_i32.offset_to_bytes(), 2147483648_u32.to_le_bytes());
    assert_eq!(i32::MAX.offset_to_bytes(), u32::MAX.to_le_bytes());

    assert_eq!(i64::MIN.offset_to_bytes(), 0_u64.to_le_bytes());
    assert_eq!(
        0_i64.offset_to_bytes(),
        9223372036854775808_u64.to_le_bytes()
    );
    assert_eq!(i64::MAX.offset_to_bytes(), u64::MAX.to_le_bytes());

    assert_eq!(i128::MIN.offset_to_bytes(), 0_u128.to_le_bytes());
    assert_eq!(
        0_i128.offset_to_bytes(),
        170141183460469231731687303715884105728_u128.to_le_bytes()
    );
    assert_eq!(i128::MAX.offset_to_bytes(), u128::MAX.to_le_bytes());
}

#[test]
fn we_can_offset_signed_integer_bytes_monotonically() {
    assert!(i8::MIN.offset_to_bytes()[0] < (-1_i8).offset_to_bytes()[0]);
    assert!((-1_i8).offset_to_bytes()[0] < 0_i8.offset_to_bytes()[0]);
    assert!(0_i8.offset_to_bytes()[0] < i8::MAX.offset_to_bytes()[0]);

    assert!(
        u16::from_le_bytes(i16::MIN.offset_to_bytes())
            < u16::from_le_bytes((-1_i16).offset_to_bytes())
    );
    assert!(
        u16::from_le_bytes((-1_i16).offset_to_bytes())
            < u16::from_le_bytes(0_i16.offset_to_bytes())
    );
    assert!(
        u16::from_le_bytes(0_i16.offset_to_bytes())
            < u16::from_le_bytes(i16::MAX.offset_to_bytes())
    );

    assert!(
        u32::from_le_bytes(i32::MIN.offset_to_bytes())
            < u32::from_le_bytes((-1_i32).offset_to_bytes())
    );
    assert!(
        u32::from_le_bytes((-1_i32).offset_to_bytes())
            < u32::from_le_bytes(0_i32.offset_to_bytes())
    );
    assert!(
        u32::from_le_bytes(0_i32.offset_to_bytes())
            < u32::from_le_bytes(i32::MAX.offset_to_bytes())
    );

    assert!(
        u64::from_le_bytes(i64::MIN.offset_to_bytes())
            < u64::from_le_bytes((-1_i64).offset_to_bytes())
    );
    assert!(
        u64::from_le_bytes((-1_i64).offset_to_bytes())
            < u64::from_le_bytes(0_i64.offset_to_bytes())
    );
    assert!(
        u64::from_le_bytes(0_i64.offset_to_bytes())
            < u64::from_le_bytes(i64::MAX.offset_to_bytes())
    );

    assert!(
        u128::from_le_bytes(i128::MIN.offset_to_bytes())
            < u128::from_le_bytes((-1_i128).offset_to_bytes())
    );
    assert!(
        u128::from_le_bytes((-1_i128).offset_to_bytes())
            < u128::from_le_bytes(0_i128.offset_to_bytes())
    );
    assert!(
        u128::from_le_bytes(0_i128.offset_to_bytes())
            < u128::from_le_bytes(i128::MAX.offset_to_bytes())
    );
}

#[test]
fn we_can_convert_unsigned_boolean_and_limbed_offsets_to_bytes() {
    assert_eq!(false.offset_to_bytes(), [0]);
    assert_eq!(true.offset_to_bytes(), [1]);
    assert_eq!(7_u8.offset_to_bytes(), [7]);
    assert_eq!(
        0x0102030405060708_u64.offset_to_bytes(),
        [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,]
    );

    let limbs = [
        0x0102030405060708_u64,
        0x1112131415161718_u64,
        0x2122232425262728_u64,
        0x3132333435363738_u64,
    ];
    let expected = [
        0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12,
        0x11, 0x28, 0x27, 0x26, 0x25, 0x24, 0x23, 0x22, 0x21, 0x38, 0x37, 0x36, 0x35, 0x34, 0x33,
        0x32, 0x31,
    ];
    assert_eq!(limbs.offset_to_bytes(), expected);
}
