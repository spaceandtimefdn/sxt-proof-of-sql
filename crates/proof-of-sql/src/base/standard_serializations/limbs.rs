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

    #[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
    struct LimbsWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn we_serialize_limbs_in_standard_big_endian_order() {
        let wrapper = LimbsWrapper {
            limbs: [
                0x0011_2233_4455_6677,
                0x8899_aabb_ccdd_eeff,
                0x1020_3040_5060_7080,
                0x90a0_b0c0_d0e0_f001,
            ],
        };
        let config = bincode::config::legacy()
            .with_fixed_int_encoding()
            .with_big_endian();

        let serialized = bincode::serde::encode_to_vec(&wrapper, config).unwrap();
        let expected = [
            wrapper.limbs[3].to_be_bytes(),
            wrapper.limbs[2].to_be_bytes(),
            wrapper.limbs[1].to_be_bytes(),
            wrapper.limbs[0].to_be_bytes(),
        ]
        .concat();
        assert_eq!(serialized, expected);

        let (deserialized, bytes_read): (LimbsWrapper, _) =
            bincode::serde::decode_from_slice(&serialized, config).unwrap();
        assert_eq!(deserialized, wrapper);
        assert_eq!(bytes_read, serialized.len());
    }
}
