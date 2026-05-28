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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct LimbWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn serialize_limbs_uses_big_endian_order() {
        let wrapper = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        let serialized = bincode::serde::encode_to_vec(wrapper, bincode::config::legacy())
            .expect("serialize limbs");
        let (encoded_limbs, consumed): ([u64; 4], usize) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::legacy())
                .expect("decode serialized limbs");

        assert_eq!(encoded_limbs, [4, 3, 2, 1]);
        assert_eq!(consumed, serialized.len());
    }

    #[test]
    fn deserialize_to_limbs_restores_little_endian_order() {
        let encoded = [4_u64, 3, 2, 1];
        let serialized = bincode::serde::encode_to_vec(encoded, bincode::config::legacy())
            .expect("serialize encoded limbs");
        let (wrapper, consumed): (LimbWrapper, usize) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::legacy())
                .expect("deserialize limb wrapper");

        assert_eq!(wrapper.limbs, [1, 2, 3, 4]);
        assert_eq!(consumed, serialized.len());
    }
}
