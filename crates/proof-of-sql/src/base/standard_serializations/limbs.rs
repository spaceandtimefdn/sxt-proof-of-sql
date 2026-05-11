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

    #[derive(Deserialize, Serialize)]
    struct LimbsJson(
        #[serde(
            deserialize_with = "deserialize_to_limbs",
            serialize_with = "serialize_limbs"
        )]
        [u64; 4],
    );

    #[test]
    fn serialize_limbs_reverses_limb_order() {
        let serialized = serde_json::to_string(&LimbsJson([1, 2, 3, 4])).unwrap();

        assert_eq!(serialized, "[4,3,2,1]");
    }

    #[test]
    fn deserialize_to_limbs_restores_internal_limb_order() {
        let deserialized: LimbsJson = serde_json::from_str("[4,3,2,1]").unwrap();

        assert_eq!(deserialized.0, [1, 2, 3, 4]);
    }
}
