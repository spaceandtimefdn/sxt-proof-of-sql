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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct LimbsContainer {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn serialize_limbs_emits_most_significant_limb_first() {
        let value = LimbsContainer {
            limbs: [0x11, 0x22, 0x33, 0x44],
        };

        let serialized = serde_json::to_string(&value).unwrap();

        assert_eq!(serialized, r#"{"limbs":[68,51,34,17]}"#);
    }

    #[test]
    fn deserialize_to_limbs_restores_internal_limb_order() {
        let deserialized: LimbsContainer =
            serde_json::from_str(r#"{"limbs":[68,51,34,17]}"#).unwrap();

        assert_eq!(
            deserialized,
            LimbsContainer {
                limbs: [0x11, 0x22, 0x33, 0x44],
            }
        );
    }

    #[test]
    fn deserialize_to_limbs_rejects_the_wrong_limb_count() {
        let err = serde_json::from_str::<LimbsContainer>(r#"{"limbs":[1,2,3]}"#).unwrap_err();

        assert!(err.is_data());
    }
}
