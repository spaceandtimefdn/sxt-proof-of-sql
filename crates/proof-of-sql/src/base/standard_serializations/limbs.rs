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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct StandardOrder {
        limbs: [u64; 4],
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct CanonicalLimbOrder {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn we_can_serialize_limbs_in_reverse_canonical_order() {
        let serialized = try_standard_binary_serialization(CanonicalLimbOrder {
            limbs: [1, 2, 3, 4],
        })
        .unwrap();
        let expected = try_standard_binary_serialization(StandardOrder {
            limbs: [4, 3, 2, 1],
        })
        .unwrap();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn we_can_deserialize_reversed_limbs_back_to_native_order() {
        let serialized = try_standard_binary_serialization(StandardOrder {
            limbs: [4, 3, 2, 1],
        })
        .unwrap();
        let (decoded, bytes_read): (CanonicalLimbOrder, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(decoded.limbs, [1, 2, 3, 4]);
        assert_eq!(bytes_read, serialized.len());
    }
}
