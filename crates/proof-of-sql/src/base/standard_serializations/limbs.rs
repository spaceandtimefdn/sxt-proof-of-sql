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
    struct LimbContainer {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn we_can_serialize_limbs_in_external_order() {
        let container = LimbContainer {
            limbs: [1, 2, 3, 4],
        };

        assert_eq!(serde_json::to_string(&container).unwrap(), r#"{"limbs":[4,3,2,1]}"#);
    }

    #[test]
    fn we_can_deserialize_limbs_to_internal_order() {
        let container: LimbContainer = serde_json::from_str(r#"{"limbs":[4,3,2,1]}"#).unwrap();

        assert_eq!(
            container,
            LimbContainer {
                limbs: [1, 2, 3, 4],
            }
        );
    }

    #[test]
    fn we_can_round_trip_limb_serialization() {
        let container = LimbContainer {
            limbs: [u64::MIN, 123, u64::MAX - 1, u64::MAX],
        };

        let serialized = serde_json::to_string(&container).unwrap();
        let deserialized: LimbContainer = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, container);
    }
}
