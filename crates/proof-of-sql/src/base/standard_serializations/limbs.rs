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
    fn serialize_limbs_reverses_limb_order() {
        let value = LimbWrapper {
            limbs: [1, 2, 3, 4],
        };

        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"limbs":[4,3,2,1]}"#
        );
    }

    #[test]
    fn deserialize_to_limbs_reverses_limb_order() {
        let value: LimbWrapper = serde_json::from_str(r#"{"limbs":[4,3,2,1]}"#).unwrap();

        assert_eq!(
            value,
            LimbWrapper {
                limbs: [1, 2, 3, 4]
            }
        );
    }

    #[test]
    fn deserialize_to_limbs_rejects_wrong_limb_count() {
        let error = serde_json::from_str::<LimbWrapper>(r#"{"limbs":[1,2,3]}"#).unwrap_err();

        assert!(error.to_string().contains("invalid length 3"));
    }
}
