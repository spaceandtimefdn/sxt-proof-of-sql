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

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct SerializedLimbs {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn serialize_limbs_reverses_limb_order() {
        let limbs = SerializedLimbs {
            limbs: [1, 2, 3, 4],
        };

        let serialized = serde_json::to_string(&limbs).unwrap();

        assert_eq!(serialized, r#"{"limbs":[4,3,2,1]}"#);
    }

    #[test]
    fn deserialize_to_limbs_reverses_wire_order() {
        let deserialized: SerializedLimbs = serde_json::from_str(r#"{"limbs":[8,6,4,2]}"#).unwrap();

        assert_eq!(
            deserialized,
            SerializedLimbs {
                limbs: [2, 4, 6, 8],
            }
        );
    }

    #[test]
    fn limbs_round_trip_through_reversed_wire_order() {
        let limbs = SerializedLimbs {
            limbs: [u64::MIN, 17, u64::MAX - 1, u64::MAX],
        };

        let serialized = serde_json::to_string(&limbs).unwrap();
        let deserialized: SerializedLimbs = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, limbs);
    }
}
