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

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct LimbWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn serializes_limbs_in_reverse_order() {
        let value = LimbWrapper {
            limbs: [0x11, 0x22, 0x33, 0x44],
        };

        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::json!({ "limbs": [0x44, 0x33, 0x22, 0x11] })
        );
    }

    #[test]
    fn deserializes_limbs_from_reverse_order() {
        let value: LimbWrapper =
            serde_json::from_value(serde_json::json!({ "limbs": [0x44, 0x33, 0x22, 0x11] }))
                .unwrap();

        assert_eq!(
            value,
            LimbWrapper {
                limbs: [0x11, 0x22, 0x33, 0x44]
            }
        );
    }

    #[test]
    fn deserialization_rejects_arrays_with_the_wrong_limb_count() {
        let err = serde_json::from_value::<LimbWrapper>(serde_json::json!({
            "limbs": [0x33, 0x22, 0x11]
        }))
        .unwrap_err();

        assert!(err.to_string().contains("invalid length 3"));
    }
}
