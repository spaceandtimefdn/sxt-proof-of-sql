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

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct LimbSerdeTest {
        #[serde(
            deserialize_with = "deserialize_to_limbs",
            serialize_with = "serialize_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn we_can_serialize_limbs_in_standard_order() {
        let value = LimbSerdeTest {
            limbs: [1, 2, 3, 4],
        };

        let serialized = serde_json::to_string(&value).unwrap();

        assert_eq!(serialized, r#"{"limbs":[4,3,2,1]}"#);
    }

    #[test]
    fn we_can_deserialize_limbs_from_standard_order() {
        let deserialized: LimbSerdeTest = serde_json::from_str(r#"{"limbs":[4,3,2,1]}"#).unwrap();

        assert_eq!(
            deserialized,
            LimbSerdeTest {
                limbs: [1, 2, 3, 4]
            }
        );
    }
}
