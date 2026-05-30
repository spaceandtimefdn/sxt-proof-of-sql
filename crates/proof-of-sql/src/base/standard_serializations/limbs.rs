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
    fn serializes_limbs_in_reverse_order() {
        let wrapper = LimbWrapper {
            limbs: [11, 22, 33, 44],
        };

        let serialized = serde_json::to_string(&wrapper).unwrap();

        assert_eq!(serialized, r#"{"limbs":[44,33,22,11]}"#);
    }

    #[test]
    fn deserializes_reversed_limbs_to_internal_order() {
        let wrapper: LimbWrapper = serde_json::from_str(r#"{"limbs":[44,33,22,11]}"#).unwrap();

        assert_eq!(
            wrapper,
            LimbWrapper {
                limbs: [11, 22, 33, 44]
            }
        );
    }
}
