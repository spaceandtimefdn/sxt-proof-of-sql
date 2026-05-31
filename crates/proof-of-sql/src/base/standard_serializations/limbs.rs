use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) fn serialize_limbs<S: Serializer>(
    limbs: &[u64; 4],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    [limbs[3], limbs[2], limbs[1], limbs[0]].serialize(serializer)
}

pub(crate) fn deserialize_to_limbs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<[u64; 4], D::Error> {
    let limbs = <[u64; 4]>::deserialize(deserializer)?;
    Ok([limbs[3], limbs[2], limbs[1], limbs[0]])
}

#[cfg(test)]
mod tests {
    use super::{deserialize_to_limbs, serialize_limbs};
    use alloc::vec::Vec;
    use bincode::config::Config;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct LimbWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    fn fixed_big_endian_config() -> impl Config {
        bincode::config::legacy()
            .with_fixed_int_encoding()
            .with_big_endian()
    }

    #[test]
    fn limb_helpers_serialize_in_big_endian_word_order() {
        let wrapper = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        let serialized =
            bincode::serde::encode_to_vec(&wrapper, fixed_big_endian_config()).unwrap();

        let mut expected = Vec::new();
        for limb in [4_u64, 3, 2, 1] {
            expected.extend_from_slice(&limb.to_be_bytes());
        }
        assert_eq!(serialized, expected);

        let (deserialized, bytes_read): (LimbWrapper, usize) =
            bincode::serde::decode_from_slice(&serialized, fixed_big_endian_config()).unwrap();
        assert_eq!(deserialized, wrapper);
        assert_eq!(bytes_read, serialized.len());
    }
}
