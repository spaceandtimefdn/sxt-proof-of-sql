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
    fn integers_are_serialized_with_fixed_width_big_endian_encoding() {
        let serialized = try_standard_binary_serialization((0x1234_u16, 0x89AB_CDEF_u32))
            .expect("tuple serialization should succeed");

        assert_eq!(serialized, [0x12, 0x34, 0x89, 0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn deserialization_reports_consumed_bytes() {
        let mut bytes = try_standard_binary_serialization((7_u16, true))
            .expect("tuple serialization should succeed");
        bytes.extend_from_slice(&[0xAA, 0xBB]);

        let (deserialized, consumed): ((u16, bool), _) =
            try_standard_binary_deserialization(&bytes).expect("tuple deserialization should pass");

        assert_eq!(deserialized, (7, true));
        assert_eq!(consumed, bytes.len() - 2);
    }

    #[test]
    fn deserialization_rejects_invalid_binary_data() {
        let error = try_standard_binary_deserialization::<(u16, bool)>(&[0x00, 0x01])
            .expect_err("truncated tuple should fail to deserialize");

        assert!(matches!(
            error,
            bincode::error::DecodeError::UnexpectedEnd { .. }
        ));
    }
}
