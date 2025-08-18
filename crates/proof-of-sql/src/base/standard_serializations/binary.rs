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
}
