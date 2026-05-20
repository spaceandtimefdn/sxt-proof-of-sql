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
    use crate::base::standard_serializations::binary::{
        try_standard_binary_deserialization, try_standard_binary_serialization,
    };
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct LimbSerde {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn we_serialize_limbs_in_reverse_order() {
        let value = LimbSerde {
            limbs: [1, 2, 3, 4],
        };

        let serialized = try_standard_binary_serialization(value).unwrap();
        let expected = [4_u64, 3, 2, 1]
            .into_iter()
            .flat_map(u64::to_be_bytes)
            .collect::<Vec<_>>();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn we_round_trip_reversed_limbs() {
        let value = LimbSerde {
            limbs: [u64::MIN, 1, u64::MAX - 1, u64::MAX],
        };
        let serialized = try_standard_binary_serialization(&value).unwrap();

        let (deserialized, bytes_read): (LimbSerde, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, value);
        assert_eq!(bytes_read, serialized.len());
    }
}
