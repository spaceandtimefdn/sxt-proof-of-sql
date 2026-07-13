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
    fn we_serialize_limbs_in_reverse_order() {
        let wrapper = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        assert_eq!(
            serde_json::to_string(&wrapper).unwrap(),
            r#"{"limbs":[4,3,2,1]}"#
        );
    }

    #[test]
    fn we_deserialize_limbs_back_to_internal_order() {
        let wrapper: LimbWrapper = serde_json::from_str(r#"{"limbs":[4,3,2,1]}"#).unwrap();

        assert_eq!(
            wrapper,
            LimbWrapper {
                limbs: [1, 2, 3, 4]
            }
        );
    }
}
