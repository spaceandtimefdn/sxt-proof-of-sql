use super::offset_to_bytes::OffsetToBytes;

#[test]
fn signed_offsets_shift_minimum_values_to_zero() {
    assert_eq!(i8::MIN.offset_to_bytes(), [0]);
    assert_eq!(i16::MIN.offset_to_bytes(), [0, 0]);
    assert_eq!(i32::MIN.offset_to_bytes(), [0, 0, 0, 0]);
    assert_eq!(i64::MIN.offset_to_bytes(), [0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(i128::MIN.offset_to_bytes(), [0; 16]);
}

#[test]
fn signed_offsets_map_zero_to_the_midpoint() {
    assert_eq!(0_i8.offset_to_bytes(), [0x80]);
    assert_eq!(0_i16.offset_to_bytes(), [0x00, 0x80]);
    assert_eq!(0_i32.offset_to_bytes(), [0x00, 0x00, 0x00, 0x80]);
    assert_eq!(
        0_i64.offset_to_bytes(),
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
    );
    assert_eq!(
        0_i128.offset_to_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]
    );
}

#[test]
fn signed_offsets_shift_maximum_values_to_all_ones() {
    assert_eq!(i8::MAX.offset_to_bytes(), [0xff]);
    assert_eq!(i16::MAX.offset_to_bytes(), [0xff, 0xff]);
    assert_eq!(i32::MAX.offset_to_bytes(), [0xff, 0xff, 0xff, 0xff]);
    assert_eq!(i64::MAX.offset_to_bytes(), [0xff; 8]);
    assert_eq!(i128::MAX.offset_to_bytes(), [0xff; 16]);
}

#[test]
fn unsigned_and_boolean_offsets_use_direct_bytes() {
    assert_eq!(false.offset_to_bytes(), [0]);
    assert_eq!(true.offset_to_bytes(), [1]);
    assert_eq!(0xab_u8.offset_to_bytes(), [0xab]);
    assert_eq!(
        0x0102_0304_0506_0708_u64.offset_to_bytes(),
        [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
    );
}

#[test]
fn u64_word_arrays_are_cast_to_little_endian_bytes() {
    let words = [
        0x0102_0304_0506_0708_u64,
        0x1112_1314_1516_1718_u64,
        0x2122_2324_2526_2728_u64,
        0x3132_3334_3536_3738_u64,
    ];
    let mut expected = [0_u8; 32];
    for (index, word) in words.iter().enumerate() {
        let start = index * 8;
        expected[start..start + 8].copy_from_slice(&word.to_le_bytes());
    }

    assert_eq!(words.offset_to_bytes(), expected);
}
