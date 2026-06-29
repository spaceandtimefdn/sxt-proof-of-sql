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
    fn we_can_serialize_limbs_in_reverse_word_order() {
        let wrapper = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        assert_eq!(
            serde_json::to_string(&wrapper).unwrap(),
            r#"{"limbs":[4,3,2,1]}"#
        );
    }

    #[test]
    fn we_can_deserialize_limbs_back_to_internal_order() {
        let wrapper: LimbWrapper = serde_json::from_str(r#"{"limbs":[40,30,20,10]}"#).unwrap();

        assert_eq!(
            wrapper,
            LimbWrapper {
                limbs: [10, 20, 30, 40],
            }
        );
    }

    #[test]
    fn we_can_round_trip_internal_limb_order() {
        let wrapper = LimbWrapper {
            limbs: [u64::MAX, 17, 0, 1 << 63],
        };

        let serialized = serde_json::to_string(&wrapper).unwrap();
        let deserialized: LimbWrapper = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, wrapper);
    }

    #[test]
    fn we_cannot_deserialize_wrong_limb_count() {
        let result = serde_json::from_str::<LimbWrapper>(r#"{"limbs":[3,2,1]}"#);

        assert!(result.is_err());
    }
}
