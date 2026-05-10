use alloc::vec::Vec;
use bincode::{
    config::Config,
    error::{DecodeError, EncodeError},
};
use serde::{Deserialize, Serialize};

fn standard_binary_config() -> impl Config {
    bincode::config::legacy()
        .with_fixed_int_encoding()
        .with_big_endian()
}

/// The standard serialization we use for our proof types
pub fn try_standard_binary_serialization(
    value_to_be_serialized: impl Serialize,
) -> Result<Vec<u8>, EncodeError> {
    bincode::serde::encode_to_vec(value_to_be_serialized, standard_binary_config())
}

/// The standard deserialization we use for our proof types
pub fn try_standard_binary_deserialization<D: for<'a> Deserialize<'a>>(
    value_to_be_deserialized: &[u8],
) -> Result<(D, usize), DecodeError> {
    bincode::serde::decode_from_slice(value_to_be_deserialized, standard_binary_config())
}

#[cfg(test)]
mod tests {
    use super::{try_standard_binary_deserialization, try_standard_binary_serialization};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct SerdeTestType {
        a: String,
        b: bool,
        c: i32,
    }

    #[test]
    fn round_trip() {
        let obj = SerdeTestType {
            a: "test".to_string(),
            b: false,
            c: 123,
        };
        let serialized = try_standard_binary_serialization(obj.clone()).unwrap();
        let (deserialized, _): (SerdeTestType, _) =
            try_standard_binary_deserialization(&serialized).unwrap();
        assert_eq!(obj, deserialized);
        let reserialized = try_standard_binary_serialization(deserialized).unwrap();
        assert_eq!(serialized, reserialized);
    }

    #[test]
    fn serializes_in_fixed_width_big_endian_order() {
        let serialized = try_standard_binary_serialization((0x1234u16, -2i16)).unwrap();

        assert_eq!(serialized, [0x12, 0x34, 0xff, 0xfe]);
    }

    #[test]
    fn deserialization_reports_consumed_byte_count_with_trailing_bytes() {
        let mut serialized = try_standard_binary_serialization(0x1234_5678u32).unwrap();
        let original_len = serialized.len();
        serialized.extend_from_slice(&[0xaa, 0xbb]);

        let (deserialized, bytes_read): (u32, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, 0x1234_5678);
        assert_eq!(bytes_read, original_len);
    }

    #[test]
    fn deserialization_rejects_invalid_boolean_encoding() {
        let err = try_standard_binary_deserialization::<bool>(&[2]).unwrap_err();

        assert!(matches!(
            err,
            bincode::error::DecodeError::InvalidBooleanValue(2)
        ));
    }
}
