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
    use bincode::error::DecodeError;
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
    fn serialization_uses_fixed_width_big_endian_integers() {
        let serialized = try_standard_binary_serialization(0x0102_0304_u32).unwrap();
        assert_eq!(serialized, [1, 2, 3, 4]);
    }

    #[test]
    fn deserialization_reports_consumed_bytes_with_trailing_payload() {
        let mut serialized = try_standard_binary_serialization(0x0102_0304_u32).unwrap();
        serialized.extend_from_slice(&[0xaa, 0xbb]);

        let (deserialized, consumed): (u32, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, 0x0102_0304);
        assert_eq!(consumed, 4);
        assert_eq!(&serialized[consumed..], &[0xaa, 0xbb]);
    }

    #[test]
    fn deserialization_rejects_incomplete_fixed_width_integer() {
        let err = try_standard_binary_deserialization::<u32>(&[1, 2, 3]).unwrap_err();
        assert!(matches!(err, DecodeError::UnexpectedEnd { .. }));
    }
}
