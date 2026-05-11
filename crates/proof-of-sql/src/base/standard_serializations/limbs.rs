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
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct LimbWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn limbs_are_serialized_in_reverse_word_order() {
        let wrapper = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        let serialized = serde_json::to_string(&wrapper).unwrap();

        assert_eq!(serialized, r#"{"limbs":[4,3,2,1]}"#);
    }

    #[test]
    fn limbs_roundtrip_back_to_the_original_layout() {
        let wrapper = LimbWrapper {
            limbs: [11, 22, 33, 44],
        };

        let serialized = serde_json::to_string(&wrapper).unwrap();
        let deserialized: LimbWrapper = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, wrapper);
    }

    #[test]
    fn deserializing_the_wrong_number_of_limbs_fails() {
        assert!(serde_json::from_str::<LimbWrapper>(r#"{"limbs":[1,2,3]}"#).is_err());
        assert!(serde_json::from_str::<LimbWrapper>(r#"{"limbs":[1,2,3,4,5]}"#).is_err());
    }
}
